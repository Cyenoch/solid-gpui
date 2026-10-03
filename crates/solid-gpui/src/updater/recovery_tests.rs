use super::*;
use ed25519_dalek::{Signer, SigningKey};
use gpui::http_client::{AsyncBody, HttpClient, Request, Response};
use std::{fs, sync::mpsc};

struct LocalRelease {
    feed: Vec<u8>,
    archive: Vec<u8>,
}

impl HttpClient for LocalRelease {
    fn user_agent(&self) -> Option<&gpui::http_client::http::HeaderValue> {
        None
    }
    fn proxy(&self) -> Option<&gpui::http_client::Url> {
        None
    }
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> futures::future::BoxFuture<'static, gpui::http_client::Result<Response<AsyncBody>>> {
        let bytes = if request.uri().path() == "/feed" {
            self.feed.clone()
        } else {
            self.archive.clone()
        };
        Box::pin(async move { Ok(Response::builder().status(200).body(bytes.into()).unwrap()) })
    }
}

struct Fixture {
    _root: tempfile::TempDir,
    config: UpdaterConfig,
    client: Arc<dyn HttpClient>,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let install = root.path().canonicalize().unwrap().join("Fixture.app");
        fs::create_dir_all(install.join("Contents/MacOS")).unwrap();
        fs::write(install.join("Contents/MacOS/fixture"), "old").unwrap();
        let mut tar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_ustar();
        header.set_mode(0o755);
        header.set_size(3);
        header.set_cksum();
        tar.append_data(
            &mut header,
            "Fixture.app/Contents/MacOS/fixture",
            &b"new"[..],
        )
        .unwrap();
        let archive = tar.into_inner().unwrap();
        let key = SigningKey::from_bytes(&[42; 32]);
        let payload = serde_json::to_vec(&serde_json::json!({
            "format": "solid-gpui-update-v1", "appId": "dev.fixture", "channel": "stable",
            "platform": acquisition::platform(), "sequence": 2, "version": "2.0.0",
            "archiveFormat": "app-tar-v1", "url": "http://127.0.0.1/artifact",
            "archiveBytes": archive.len(), "sha256": acquisition::hex(&sha2::Sha256::digest(&archive)),
            "bundle": "Fixture.app", "executable": "Contents/MacOS/fixture"
        })).unwrap();
        let feed = serde_json::to_vec(&serde_json::json!({
            "payload": acquisition::hex(&payload), "signature": acquisition::hex(&key.sign(&payload).to_bytes())
        })).unwrap();
        Self {
            _root: root,
            config: UpdaterConfig {
                app_id: "dev.fixture".into(),
                channel: "stable".into(),
                feed_url: "http://127.0.0.1/feed".into(),
                public_key: key.verifying_key().to_bytes(),
                current_sequence: 1,
                install_path: install,
                executable: "Contents/MacOS/fixture".into(),
                max_archive_bytes: 65536,
                max_unpacked_bytes: 65536,
                max_entries: 20,
                timeout: Duration::from_secs(2),
                allow_loopback_http: true,
            },
            client: Arc::new(LocalRelease { feed, archive }),
        }
    }
    fn open(&self) -> Arc<SignedUpdater> {
        Arc::new(SignedUpdater::new(self.config.clone(), self.client.clone()).unwrap())
    }
    fn contents(&self) -> String {
        fs::read_to_string(self.config.install_path.join("Contents/MacOS/fixture")).unwrap()
    }
    fn record(&self) -> PathBuf {
        self.config
            .install_path
            .with_file_name(".Fixture.app.solid-update")
            .join("record.json")
    }
}

fn install(service: &SignedUpdater) -> Result<UpdateStatus, String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let _entered = runtime.enter();
    let offer = service.check(NativeCallContext::default())?.unwrap();
    service.install(
        UpdateRequest { token: offer.token },
        NativeCallContext::default(),
    )
}

#[test]
fn dropping_an_updater_releases_ownership_with_a_shared_file_descriptor() {
    let fixture = Fixture::new();
    let service = fixture.open();
    let shared = service._lock.file.try_clone().unwrap();
    assert!(SignedUpdater::new(fixture.config.clone(), fixture.client.clone()).is_err());
    drop(service);
    let reopened = fixture.open();
    drop(shared);
    assert!(SignedUpdater::new(fixture.config.clone(), fixture.client.clone()).is_err());
    drop(reopened);
    fixture.open();
}

#[test]
fn status_waits_for_a_staged_install_instead_of_reading_incomplete_bundle_identities() {
    let fixture = Fixture::new();
    let service = fixture.open();
    let (staged_tx, staged_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let installer = service.clone();
    let installation = std::thread::spawn(move || {
        test_io::pause("before-exchange", staged_tx, release_rx);
        install(&installer)
    });
    staged_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(fixture.contents(), "old");
    assert!(fixture.record().exists());
    let (status_tx, status_rx) = mpsc::channel();
    let observer = service.clone();
    let observation = std::thread::spawn(move || {
        status_tx.send(observer.status()).unwrap();
    });
    let before_commit = status_rx.recv_timeout(Duration::from_millis(100));
    release_tx.send(()).unwrap();
    assert!(installation.join().unwrap().unwrap().rollback_available);
    observation.join().unwrap();
    assert!(
        before_commit.is_err(),
        "status must wait while the installation mutex is owned"
    );
    let status = status_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .unwrap();
    assert!(status.pending_restart && status.rollback_available);
    assert_eq!(fixture.contents(), "new");
}

#[test]
fn cleanup_failures_retain_signed_identity_and_restart_finishes_the_selected_install() {
    for confirm in [false, true] {
        for point in [
            "cleanup-phase-sync",
            "remove-candidate",
            "partial-candidate-remove",
            "cleanup-sync",
            "remove-record",
            "record-remove-sync",
        ] {
            let mut fixture = Fixture::new();
            let mut service = fixture.open();
            install(&service).unwrap();
            if confirm {
                drop(service);
                fixture.config.current_sequence = 2;
                service = fixture.open();
            }
            test_io::failures(&[point]);
            let result = if confirm {
                service.confirm(NativeCallContext::default())
            } else {
                service.rollback(NativeCallContext::default())
            };
            assert!(result.err().unwrap().contains(point));
            test_io::exhausted();
            assert_eq!(
                fixture.record().exists(),
                point != "record-remove-sync",
                "signed identity must survive until candidate deletion is durable: {point}"
            );
            assert!(!service.status().unwrap().rollback_available);
            assert_eq!(fixture.contents(), if confirm { "new" } else { "old" });
            drop(service);
            let reopened = fixture.open();
            assert!(!reopened.status().unwrap().rollback_available);
            assert!(!fixture.record().exists());
            assert_eq!(fixture.contents(), if confirm { "new" } else { "old" });
        }
    }
}

#[test]
fn post_swap_rollback_failure_retains_backup_and_restart_completes_restore() {
    for point in ["restore-exchange", "restore-sync"] {
        let fixture = Fixture::new();
        let mut service = fixture.open();
        test_io::failures(&["post-exchange-sync", point]);
        let error = install(&service).err().unwrap();
        assert!(error.contains("rollback failed"));
        test_io::exhausted();
        assert_eq!(
            fixture.contents(),
            if point == "restore-exchange" {
                "new"
            } else {
                "old"
            }
        );
        assert!(fixture.record().exists());
        assert_eq!(
            service.status().unwrap().rollback_available,
            point == "restore-exchange"
        );
        if point == "restore-exchange" {
            Arc::get_mut(&mut service).unwrap().config.current_sequence = 2;
            assert!(
                service
                    .confirm(NativeCallContext::default())
                    .err()
                    .unwrap()
                    .contains("rollback decision")
            );
        }
        drop(service);
        let reopened = fixture.open();
        assert_eq!(fixture.contents(), "old");
        assert!(!reopened.status().unwrap().rollback_available);
        assert!(!fixture.record().exists());
    }
}

#[test]
fn precommit_inspection_and_sync_failure_preserve_the_original_then_restart_cleans_up() {
    let fixture = Fixture::new();
    let service = fixture.open();
    test_io::failures(&["record-write-sync"]);
    assert!(
        install(&service)
            .err()
            .unwrap()
            .contains("record-write-sync")
    );
    test_io::exhausted();
    assert_eq!(fixture.contents(), "old");
    assert!(fixture.record().exists());
    assert!(!service.status().unwrap().rollback_available);
    drop(service);
    let reopened = fixture.open();
    assert_eq!(fixture.contents(), "old");
    assert!(!fixture.record().exists());
    assert!(!reopened.status().unwrap().rollback_available);
}

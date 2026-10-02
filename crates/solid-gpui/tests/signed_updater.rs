#![cfg(all(feature = "signed-updater", not(target_family = "wasm")))]

#[cfg(target_os = "macos")]
mod macos {

    use ed25519_dalek::{Signer, SigningKey};
    use sha2::{Digest, Sha256};
    use solid_gpui::ExtensionRegistry;
    use solid_gpui::updater::{SignedUpdater, UpdaterConfig, native_module};
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::Arc,
        time::Duration,
    };

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    struct Fixture {
        _root: tempfile::TempDir,
        service: Arc<SignedUpdater>,
        install: std::path::PathBuf,
        config: UpdaterConfig,
    }

    fn fixture(
        archive: Vec<u8>,
        mutate: impl FnOnce(&mut serde_json::Value),
        corrupt: bool,
    ) -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let install = root.path().canonicalize().unwrap().join("Fixture.app");
        std::fs::create_dir_all(install.join("Contents/MacOS")).unwrap();
        std::fs::write(install.join("Contents/MacOS/fixture"), "old").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let key = SigningKey::from_bytes(&[42; 32]);
        let mut manifest = serde_json::json!({
            "format": "solid-gpui-update-v1", "appId": "dev.fixture", "channel": "stable",
            "platform": format!("macos-{}", std::env::consts::ARCH), "sequence": 2, "version": "2.0.0",
            "archiveFormat": "app-tar-v1", "url": format!("{origin}/artifact"),
            "archiveBytes": archive.len(), "sha256": hex(&Sha256::digest(&archive)),
            "bundle": "Fixture.app", "executable": "Contents/MacOS/fixture"
        });
        mutate(&mut manifest);
        let payload = serde_json::to_vec(&manifest).unwrap();
        let mut signature = key.sign(&payload).to_bytes();
        if corrupt {
            signature[0] ^= 1;
        }
        let feed = serde_json::to_vec(
            &serde_json::json!({"payload": hex(&payload), "signature": hex(&signature)}),
        )
        .unwrap();
        std::thread::spawn(move || {
            // Nonblocking accept keeps fixture workers bounded even on early rejection.
            listener.set_nonblocking(true).unwrap();
            let end = std::time::Instant::now() + Duration::from_secs(3);
            let mut served = 0;
            while served < 3 && std::time::Instant::now() < end {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .unwrap();
                        let mut request = [0; 4096];
                        let n = stream.read(&mut request).unwrap();
                        let body =
                            if String::from_utf8_lossy(&request[..n]).starts_with("GET /feed ") {
                                &feed
                            } else {
                                &archive
                            };
                        let _ = write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(body);
                        served += 1;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        });
        let config = UpdaterConfig {
            app_id: "dev.fixture".into(),
            channel: "stable".into(),
            feed_url: format!("{origin}/feed"),
            public_key: key.verifying_key().to_bytes(),
            current_sequence: 1,
            install_path: install.clone(),
            executable: "Contents/MacOS/fixture".into(),
            max_archive_bytes: 65536,
            max_unpacked_bytes: 65536,
            max_entries: 20,
            timeout: Duration::from_secs(2),
            allow_loopback_http: true,
        };
        let service = Arc::new(
            SignedUpdater::new(
                config.clone(),
                Arc::new(reqwest_client::ReqwestClient::new()),
            )
            .unwrap(),
        );
        Fixture {
            _root: root,
            service,
            install,
            config,
        }
    }

    fn archive() -> Vec<u8> {
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
        tar.into_inner().unwrap()
    }

    fn invoke(
        service: Arc<SignedUpdater>,
        name: &str,
        value: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let definition = native_module(Some(service));
        let command = definition.command_id(name).unwrap();
        let module = definition
            .native_module(definition.id(), definition.digest())
            .unwrap();
        let result = module.invoke(
            command,
            &solid_gpui::native::encode_native_request(definition.build_digest(), &value).unwrap(),
        )?;
        Ok(serde_json::from_slice(&result).unwrap())
    }

    #[test]
    fn signed_local_release_installs_atomically_and_rolls_back_through_native_commands() {
        let f = fixture(archive(), |_| {}, false);
        let offer = invoke(f.service.clone(), "checkUpdate", serde_json::Value::Null).unwrap();
        let result = invoke(
            f.service.clone(),
            "installUpdate",
            serde_json::json!({"token": offer["token"]}),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
            "new"
        );
        assert_eq!(result["pendingRestart"], true);
        assert_eq!(result["rollbackAvailable"], true);
        let rolled_back =
            invoke(f.service.clone(), "rollbackUpdate", serde_json::Value::Null).unwrap();
        assert_eq!(rolled_back["pendingRestart"], true);
        assert_eq!(rolled_back["rollbackAvailable"], false);
        assert_eq!(
            std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
            "old"
        );
    }

    #[test]
    fn bad_signature_identity_and_artifact_leave_the_old_application_intact() {
        for (mutation, corrupt) in [(0, true), (1, false), (2, false), (3, false), (4, false)] {
            let f = fixture(
                archive(),
                |manifest| match mutation {
                    1 => manifest["channel"] = "preview".into(),
                    2 => manifest["platform"] = "linux-x86_64".into(),
                    3 => manifest["sha256"] = "00".repeat(32).into(),
                    4 => manifest["appId"] = "other.app".into(),
                    _ => (),
                },
                corrupt,
            );
            let offer = invoke(f.service.clone(), "checkUpdate", serde_json::Value::Null);
            if mutation == 3 {
                assert!(
                    invoke(
                        f.service.clone(),
                        "installUpdate",
                        serde_json::json!({"token": offer.unwrap()["token"]})
                    )
                    .unwrap_err()
                    .contains("SHA-256")
                );
            } else {
                assert!(offer.is_err());
            }
            assert_eq!(
                std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
                "old"
            );
            assert!(!f.service.status().unwrap().rollback_available);
            assert!(
                !f.install
                    .with_file_name(".Fixture.app.solid-update")
                    .join("candidate")
                    .exists()
            );
        }
    }

    #[test]
    fn authenticated_archive_cannot_escape_or_install_links_or_exceed_limits() {
        for threat in ["traversal", "symlink", "hardlink", "size", "duplicate"] {
            let mut builder = tar::Builder::new(Vec::new());
            let mut header = tar::Header::new_ustar();
            header.set_mode(0o755);
            header.set_size(3);
            header
                .set_path("Fixture.app/Contents/MacOS/fixture")
                .unwrap();
            match threat {
                "traversal" => {
                    let bytes = header.as_mut_bytes();
                    bytes[..100].fill(0);
                    bytes[..10].copy_from_slice(b"../escaped");
                }
                "symlink" | "hardlink" => {
                    header.set_entry_type(if threat == "symlink" {
                        tar::EntryType::Symlink
                    } else {
                        tar::EntryType::Link
                    });
                    header.set_link_name("/outside").unwrap();
                }
                "size" => header.set_size(65537),
                _ => (),
            }
            header.set_cksum();
            builder.append(&header, &b"new"[..]).unwrap();
            if threat == "duplicate" {
                builder.append(&header, &b"new"[..]).unwrap();
            }
            let f = fixture(builder.into_inner().unwrap(), |_| {}, false);
            let offer = invoke(f.service.clone(), "checkUpdate", serde_json::Value::Null).unwrap();
            assert!(
                invoke(
                    f.service.clone(),
                    "installUpdate",
                    serde_json::json!({"token": offer["token"]})
                )
                .is_err(),
                "{threat}"
            );
            assert_eq!(
                std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
                "old"
            );
            assert!(!f._root.path().join("escaped").exists());
        }
    }

    #[test]
    fn pending_rollback_survives_owner_restart_and_only_new_release_can_confirm() {
        let f = fixture(archive(), |_| {}, false);
        let offer = invoke(f.service.clone(), "checkUpdate", serde_json::Value::Null).unwrap();
        assert!(
            invoke(
                f.service.clone(),
                "installUpdate",
                serde_json::json!({"token": "untrusted"})
            )
            .is_err()
        );
        invoke(
            f.service.clone(),
            "installUpdate",
            serde_json::json!({"token": offer["token"]}),
        )
        .unwrap();
        assert!(
            invoke(f.service.clone(), "confirmUpdate", serde_json::Value::Null)
                .unwrap_err()
                .contains("restarted")
        );
        let Fixture {
            _root,
            service,
            install,
            mut config,
        } = f;
        drop(service);
        let reopened = Arc::new(
            SignedUpdater::new(
                config.clone(),
                Arc::new(reqwest_client::ReqwestClient::new()),
            )
            .unwrap(),
        );
        assert!(reopened.status().unwrap().rollback_available);
        drop(reopened);
        config.current_sequence = 2;
        let reopened = Arc::new(
            SignedUpdater::new(config, Arc::new(reqwest_client::ReqwestClient::new())).unwrap(),
        );
        invoke(reopened.clone(), "confirmUpdate", serde_json::Value::Null).unwrap();
        assert!(!reopened.status().unwrap().rollback_available);
        assert_eq!(
            std::fs::read_to_string(install.join("Contents/MacOS/fixture")).unwrap(),
            "new"
        );
    }

    #[test]
    fn cancellation_aborts_a_stalled_download_and_releases_service_ownership() {
        let f = fixture(archive(), |_| {}, false);
        let mut config = f.config.clone();
        drop(f.service);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        config.feed_url = format!("http://{}/feed", listener.local_addr().unwrap());
        config.timeout = Duration::from_millis(500);
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).unwrap() > 0);
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        });
        let service = Arc::new(
            SignedUpdater::new(config, Arc::new(reqwest_client::ReqwestClient::new())).unwrap(),
        );
        let definition = native_module(Some(service.clone()));
        let command = definition.command_id("checkUpdate").unwrap();
        let module = definition
            .native_module(definition.id(), definition.digest())
            .unwrap();
        let call = module.invoke_async(
            command,
            solid_gpui::native::encode_native_request(definition.build_digest(), &()).unwrap(),
        );
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(call);
        let end = std::time::Instant::now() + Duration::from_secs(1);
        loop {
            // After cancellation a new invocation can acquire the service. A malformed
            // token then fails without making a second HTTP request.
            let result = invoke(
                service.clone(),
                "installUpdate",
                serde_json::json!({"token": "stale"}),
            );
            if result.unwrap_err().contains("check for an update") {
                break;
            }
            assert!(std::time::Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        release_tx.send(()).unwrap();
        assert_eq!(
            std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
            "old"
        );
        assert!(!service.status().unwrap().rollback_available);
    }

    #[test]
    fn oversized_or_redirected_feed_and_symlink_install_are_rejected() {
        let f = fixture(archive(), |_| {}, false);
        let mut config = f.config.clone();
        drop(f.service);
        let alias = f._root.path().canonicalize().unwrap().join("Alias.app");
        std::os::unix::fs::symlink(&f.install, &alias).unwrap();
        config.install_path = alias;
        assert!(
            SignedUpdater::new(
                config.clone(),
                Arc::new(reqwest_client::ReqwestClient::new())
            )
            .err()
            .unwrap()
            .contains("symlinks")
        );
        config.install_path = f.install.clone();
        for (status, body) in [("200 OK", "x".repeat(32769)), ("302 Found", String::new())] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            config.feed_url = format!("http://{}/feed", listener.local_addr().unwrap());
            std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                assert!(stream.read(&mut request).unwrap() > 0);
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\nLocation: https://untrusted.invalid\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            });
            let service = Arc::new(
                SignedUpdater::new(
                    config.clone(),
                    Arc::new(reqwest_client::ReqwestClient::new()),
                )
                .unwrap(),
            );
            assert!(invoke(service, "checkUpdate", serde_json::Value::Null).is_err());
        }
    }

    #[test]
    fn failed_atomic_exchange_preserves_the_old_bundle_and_allows_retry() {
        use std::os::unix::fs::PermissionsExt;
        let f = fixture(archive(), |_| {}, false);
        let offer = invoke(f.service.clone(), "checkUpdate", serde_json::Value::Null).unwrap();
        let parent = f.install.parent().unwrap();
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o500)).unwrap();
        let result = invoke(
            f.service.clone(),
            "installUpdate",
            serde_json::json!({"token": offer["token"]}),
        );
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            result
                .unwrap_err()
                .contains("atomic bundle exchange failed")
        );
        assert_eq!(
            std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
            "old"
        );
        assert!(!f.service.status().unwrap().rollback_available);
        assert!(!parent.join(".Fixture.app.solid-update/candidate").exists());
        invoke(
            f.service.clone(),
            "installUpdate",
            serde_json::json!({"token": offer["token"]}),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(f.install.join("Contents/MacOS/fixture")).unwrap(),
            "new"
        );
    }
}

#[test]
fn disabled_updater_exports_the_real_contract_without_install_authority() {
    use solid_gpui::ExtensionRegistry;
    let module = solid_gpui::updater::native_module(None);
    let bindings = module.typescript().unwrap();
    assert!(bindings.contains("checkUpdate"));
    assert!(bindings.contains("installUpdate"));
    assert!(bindings.contains("rollbackUpdate"));
    assert!(bindings.contains("confirmUpdate"));
    assert!(bindings.contains("updateStatus"));
    let command = module.command_id("installUpdate").unwrap();
    let dispatcher = module.native_module(module.id(), module.digest()).unwrap();
    assert_eq!(
        dispatcher
            .invoke(
                command,
                &solid_gpui::native::encode_native_request(
                    module.build_digest(),
                    &serde_json::json!({"token":"untrusted"})
                )
                .unwrap()
            )
            .unwrap_err(),
        "signed updater is disabled"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_native_platforms_refuse_install_authority_before_touching_paths() {
    use solid_gpui::updater::{SignedUpdater, UpdaterConfig};
    let result = SignedUpdater::new(
        UpdaterConfig {
            app_id: "fixture".into(),
            channel: "stable".into(),
            feed_url: "https://example.invalid".into(),
            public_key: [0; 32],
            current_sequence: 1,
            install_path: "/never-touch-this.app".into(),
            executable: "Contents/MacOS/fixture".into(),
            max_archive_bytes: 1,
            max_unpacked_bytes: 1,
            max_entries: 1,
            timeout: std::time::Duration::from_secs(1),
            allow_loopback_http: false,
        },
        std::sync::Arc::new(reqwest_client::ReqwestClient::new()),
    );
    assert_eq!(
        result.err().unwrap(),
        "signed update installation is supported only on macOS"
    );
}

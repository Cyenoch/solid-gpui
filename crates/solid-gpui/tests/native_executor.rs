use solid_gpui::{
    ExtensionRegistry,
    native::{CommandDefinition, ModuleDefinition, NativeModules},
};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[test]
fn ordinary_callers_can_run_tokio_timers_and_network_io() {
    let definition = ModuleDefinition::new(
        "io",
        vec![],
        vec![CommandDefinition::asynchronous(
            "roundtrip",
            |(): (), _context| async {
                tokio::time::timeout(Duration::from_secs(3), async {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    let listener = TcpListener::bind("127.0.0.1:0").await?;
                    let address = listener.local_addr()?;
                    let server = async {
                        let (mut socket, _) = listener.accept().await?;
                        let mut bytes = [0; 4];
                        socket.read_exact(&mut bytes).await?;
                        socket.write_all(&bytes).await?;
                        Ok::<_, std::io::Error>(())
                    };
                    let client = async {
                        let mut socket = TcpStream::connect(address).await?;
                        socket.write_all(b"ping").await?;
                        let mut bytes = [0; 4];
                        socket.read_exact(&mut bytes).await?;
                        Ok::<_, std::io::Error>(String::from_utf8(bytes.to_vec()).unwrap())
                    };
                    let (_, reply) = futures::try_join!(server, client)?;
                    Ok::<_, std::io::Error>(reply)
                })
                .await
                .map_err(|error| error.to_string())?
                .map_err(|error| error.to_string())
            },
        )],
    );
    let module = definition
        .native_module(definition.id(), definition.digest())
        .unwrap();
    assert!(tokio::runtime::Handle::try_current().is_err());
    assert_eq!(module.invoke(1, b"null").unwrap(), br#""ping""#);
    assert_eq!(
        futures::executor::block_on(module.invoke_async(1, b"null".to_vec())).unwrap(),
        br#""ping""#
    );
}

#[test]
fn composed_modules_share_one_runtime_and_panics_are_request_errors() {
    let make = |name| {
        ModuleDefinition::new(
            name,
            vec![],
            vec![
                CommandDefinition::asynchronous("runtime", |(): (), _context| async {
                    Ok(tokio::runtime::Handle::current().id().to_string())
                }),
                CommandDefinition::asynchronous("asyncPanic", |(): (), _context| async {
                    panic!("async failure");
                    #[allow(unreachable_code)]
                    Ok(())
                }),
                CommandDefinition::sync("syncPanic", |(): (), _context| {
                    panic!("sync failure");
                    #[allow(unreachable_code)]
                    Ok(())
                }),
            ],
        )
    };
    let first = make("first");
    let second = make("second");
    let ids = [(first.id(), first.digest()), (second.id(), second.digest())];
    let registry = NativeModules::new(vec![first, second]);
    let modules = ids.map(|(id, digest)| registry.native_module(id, digest).unwrap());
    for module in &modules {
        assert_eq!(
            module.invoke(1, b"null").unwrap_err(),
            "native command panicked: async failure"
        );
        assert_eq!(
            module.invoke(3, b"null").unwrap_err(),
            "native command panicked: sync failure"
        );
    }
    assert_eq!(
        modules[0].invoke(2, b"null").unwrap(),
        modules[1].invoke(2, b"null").unwrap()
    );
}

#![cfg_attr(not(target_os = "linux"), no_std)]

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use hex::encode;
    use hyper;
    use hyper::client::connect::HttpConnector;
    use ledger_apdu::APDUCommand;
    use speculos_api::apis;
    use speculos_api::apis::DefaultApi;
    use speculos_api::models::button::*;
    use speculos_api::models::*;
    use std::future::Future;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::process::Command;
    use tokio::test;
    use tokio::time::{sleep, Duration};
    use tokio_retry::strategy::FixedInterval;
    use tokio_retry::Retry;

    /*async fn test_build() {
        Command::new("cargo").arg("build").status().await.expect("Failed to execute speculos");
    }*/

    static DID_BUILD: AtomicBool = AtomicBool::new(false);

    async fn with_speculos<F, Fut, O>(f: F) -> O
    where
        F: Fn(apis::DefaultApiClient<HttpConnector>) -> Fut,
        Fut: Future<Output = O>,
    {
        if !DID_BUILD.load(Ordering::Relaxed) {
            Command::new("cargo")
                .args(["build", "-Z", "build-std=core", "--features", "speculos"])
                .status()
                .await
                .expect("Build failed");
            DID_BUILD.store(true, Ordering::Relaxed);
        }
        let _speculos = Command::new("speculos")
            .args([
                "./target/thumbv6m-none-eabi/debug/rust-app",
                "--display",
                "headless",
            ])
            .kill_on_drop(true)
            .spawn()
            .expect("Failed to execute speculos");

        let raw_client = hyper::client::Client::new();
        let client = apis::DefaultApiClient::new(std::rc::Rc::new(
            apis::configuration::Configuration::new(raw_client),
        ));

        let strat = FixedInterval::from_millis(100);
        match Retry::spawn(strat, || async {
            let a = client.events_delete().await;
            a
        })
        .await
        {
            Ok(_) => {}
            Err(_) => {
                panic!("failed to delete previous events");
            }
        }

        let rv = f(client).await;

        rv
    }

    #[test]
    async fn test_provide_pubkey() {
        with_speculos(|client| async move {
            let payload = vec!(0x01,0x00,0x00,0x00,0x00);
            let provide_pubkey = APDUCommand {
                cla: 0,
                ins: 2,
                p1: 0,
                p2: 0,
                data: payload
            };

            let res_async = client.apdu_post(Apdu::new(encode(provide_pubkey.serialize())));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                Some::<()>(())
            };
            let (res, _) = futures::join!(res_async, btns);

            assert_eq!(res.ok(), Some(Apdu { data: "046f760e57383e3b5900f7c23b78a424e74bebbe9b7b46316da7c0b4b9c2c9301c0c076310eda30506141dd47c2d0a8a1d7ca2542482926ae23b781546193b96169000".to_string() }));
            client.events_delete().await.ok()?;
            Some(())

        }).await;
        ()
    }

    #[test]
    async fn test_provide_pubkey_twice() {
        with_speculos(|client| async move {
            let payload = vec!(0x01,0x00,0x00,0x00,0x00);
            let provide_pubkey = APDUCommand {
                cla: 0,
                ins: 2,
                p1: 0,
                p2: 0,
                data: payload
            };

            let res_async = client.apdu_post(Apdu::new(encode(provide_pubkey.serialize())));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                Some(())
            };
            let (res, _) = futures::join!(res_async, btns);

            assert_eq!(res.ok(), Some(Apdu { data: "046f760e57383e3b5900f7c23b78a424e74bebbe9b7b46316da7c0b4b9c2c9301c0c076310eda30506141dd47c2d0a8a1d7ca2542482926ae23b781546193b96169000".to_string() }));

            let payload_2 = vec!(0x02,  0x00,0x00,0x00,0x00,  0x00, 0x01, 0x00, 0x00);
            let provide_pubkey_2 = APDUCommand {
                cla: 0,
                ins: 2,
                p1: 0,
                p2: 0,
                data: payload_2
            };

            let res_async_2 = client.apdu_post(Apdu::new(encode(provide_pubkey_2.serialize())));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                Some(())
            };
            let (res_2, _) = futures::join!(res_async_2, btns);

            assert_eq!(res_2.ok(), Some(Apdu { data: "04b90248e0ca25f494e709105e82624145dae654449d81fb557f6b764d1461940080139785d8fc752bb070751f1ef3ff4723119fb6ba1ab14c01a8be8f975311649000".to_string() }));
            client.events_delete().await.ok()?;
            Some(())
        }).await;
        ()
    }
}

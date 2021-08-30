#![cfg_attr(not(target_os="linux"), no_std)]

#[cfg(all(test, target_os="linux"))]
mod tests {
//    use async_std::task;
//    use async_std::process::Command;
//    use transport_speculos::*;
    use tokio::process::Command;
    use speculos_api::apis;
    use speculos_api::apis::DefaultApi;
    use tokio::test;
    use hyper;
    use hyper::client::connect::HttpConnector;
    use speculos_api::models::*;
    use speculos_api::models::button::*;
    use std::future::Future;
    use tokio::time::{sleep, Duration};
    use hex::encode;
    use tokio_retry::Retry;
    use tokio_retry::strategy::FixedInterval;
    // use hex::decode;

    /*async fn test_build() {
        Command::new("cargo").arg("build").status().await.expect("Failed to execute speculos");
    }*/


    async fn with_speculos<F, Fut, O> (f: F) -> O where
        F: Fn(apis::DefaultApiClient<HttpConnector>) -> Fut,
        Fut: Future<Output = O>,
        {
        
        let mut speculos = Command::new("cargo").args(["run", "--features", "speculos", "-Z", "build-std=core"]).spawn().expect("Failed to execute speculos"); //, "--", "--apdu-port", "9999", "--automation-port", "9998", "--button-port", "9997"]).spawn().expect("Failed to execute speculos");

        let raw_client = hyper::client::Client::new();
        let client = apis::DefaultApiClient::new(std::rc::Rc::new(apis::configuration::Configuration::new(raw_client)));

        let strat = FixedInterval::from_millis(100);
        match Retry::spawn(strat, || async { let a = client.events_delete().await; a }).await {
            Ok(_) => {}
            Err(_) => { panic!("Blarg"); }
        }

        let rv = f(client).await;

        speculos.kill().await.expect("Failed to kill speculos");

        rv
    }

    #[test]
    async fn test_provide_pubkey() {
        with_speculos(|client| async move {
            let payload = vec!(0x01,0x00,0x00,0x00,0x00);
            let mut apdu = vec!(0x00, 0x02, payload.len() as u8, 0x00);
            apdu.extend(payload);
            /*let provide_pubkey = APDUCommand {
                cla: 0,
                ins: 2,
                p1: payload.len() as u8,
                p2: 0,
                data: payload
            };*/

            let res_async = client.apdu_post(Apdu::new(encode(apdu)));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                Some::<()>(())
            };
            let (res, _) = futures::join!(res_async, btns);

            assert_eq!(res.ok(), Some(Apdu { data: "046f760e57383e3b5900f7c23b78a424e74bebbe9b7b46316da7c0b4b9c2c9301c0c076310eda30506141dd47c2d0a8a1d7ca2542482926ae23b781546193b96169000".to_string() }));
            //println!("{:?}\n", client.events_get(None).await);
            //assert_eq!(1,2);
            //assert_eq!(client.events_get(None).await.ok().unwrap(), "");
            client.events_delete().await.ok()?;
            Some(())

        }).await;
        ()
    }
    

    #[test]
    async fn test_provide_pubkey_twice() {
        with_speculos(|client| async move {
            let payload = vec!(0x01,0x00,0x00,0x00,0x00);
            let mut apdu = vec!(0x00, 0x02, payload.len() as u8, 0x00);
            apdu.extend(payload);
            /*let provide_pubkey = APDUCommand {
                cla: 0,
                ins: 2,
                p1: payload.len() as u8,
                p2: 0,
                data: payload
            };*/

            let res_async = client.apdu_post(Apdu::new(encode(apdu)));

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
            let mut apdu_2 = vec!(0x00, 0x02, payload_2.len() as u8, 0x00);
            apdu_2.extend(payload_2);
            let res_async_2 = client.apdu_post(Apdu::new(encode(apdu_2)));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await.ok()?;
                Some(())
            };
            let (res_2, _) = futures::join!(res_async_2, btns);

            assert_eq!(res_2.ok(), Some(Apdu { data: "04b90248e0ca25f494e709105e82624145dae654449d81fb557f6b764d1461940080139785d8fc752bb070751f1ef3ff4723119fb6ba1ab14c01a8be8f975311649000".to_string() }));
            //println!("{:?}\n", client.events_get(None).await);
            //assert_eq!(1,2);
            //assert_eq!(client.events_get(None).await.ok().unwrap(), "");
            client.events_delete().await.ok()?;
            Some(())
        }).await;
        ()
    }

}


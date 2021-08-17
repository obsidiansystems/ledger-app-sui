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
    use ledger_apdu::*;
    use hex::encode;
    use log;
    use tokio_retry::Retry;
    use tokio_retry::strategy::FixedInterval;
    // use hex::decode;

    async fn test_build() {
        Command::new("cargo").arg("build").status().await.expect("Failed to execute speculos");
    }


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

    // #[test]
    async fn test_nothing() {
        with_speculos(|client| async move {

            let exit_cmd = APDUCommand {
                cla: 0,
                ins: 0xfe,
                p1: 0,
                p2: 0,
                data: Vec::new()
            };

            let res = client.apdu_post(Apdu::new(encode(exit_cmd.serialize()))).await;

            client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.1) }).await;
            println!("{:?}",res);
            assert_eq!(res.ok(), Some(Apdu { data: "62c5fa0b25185f1ac43c3563f90577d804a09dbd266c64561cd6c7254a6e7fcd9000".to_string() }));

        }).await;
        ()
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

            use tokio::task;
            let res_async = client.apdu_post(Apdu::new(encode(apdu)));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
            };
            let (res, _) = futures::join!(res_async, btns);

            assert_eq!(res.ok(), Some(Apdu { data: "b0f212dd8651a02ab7e2ea38a718a2547b04efb4e31c9a42d5676bf41378d9e39000".to_string() }));
            //println!("{:?}\n", client.events_get(None).await);
            //assert_eq!(1,2);
            //assert_eq!(client.events_get(None).await.ok().unwrap(), "");
            client.events_delete().await;

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
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
            };
            let (res, _) = futures::join!(res_async, btns);
            
            assert_eq!(res.ok(), Some(Apdu { data: "b0f212dd8651a02ab7e2ea38a718a2547b04efb4e31c9a42d5676bf41378d9e39000".to_string() }));

            let payload_2 = vec!(0x01,0x00,0x00,0x00,0x00);
            let mut apdu_2 = vec!(0x00, 0x02, payload_2.len() as u8, 0x00);
            apdu_2.extend(payload_2);
            let res_async_2 = client.apdu_post(Apdu::new(encode(apdu_2)));

            let btns = async {
                sleep(Duration::from_millis(2000)).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Right, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
                client.button_button_post(ButtonName::Both, Button { action: Action::PressAndRelease, delay: Some(0.5) }).await;
            };
            let (res_2, _) = futures::join!(res_async_2, btns);

            assert_eq!(res_2.ok(), Some(Apdu { data: "b0f212dd8651a02ab7e2ea38a718a2547b04efb4e31c9a42d5676bf41378d9e39000".to_string() }));
            //println!("{:?}\n", client.events_get(None).await);
            //assert_eq!(1,2);
            //assert_eq!(client.events_get(None).await.ok().unwrap(), "");
            client.events_delete().await;

        }).await;
        ()
    }

    // use rust_app;
    use ledger_parser_combinators::core_parsers;
    use ledger_parser_combinators::core_parsers::{ DArray, Byte, U32 };
    use ledger_parser_combinators::endianness::*;
    use ledger_parser_combinators::forward_parser::ForwardParser;
    use arrayvec;
    #[test]
    async fn test_pubkey_parser() {
        let parser = core_parsers::Action { 
            sub: DArray::<_,_,10>(Byte, U32::< { Endianness::Little }>),
            f: |path| {
                assert_eq!(path.as_slice(), &[0][..]);
                println!("Path: {:?}", path);
                ((), None)
            }
        };

        let mut state = parser.init_method();
        let data = vec!(0x01,0x00,0x00,0x00, 0x00);
        let rv = parser.parse(&mut state, &data);
        assert_eq!(rv, Ok(((),&[][..])));
    }
}

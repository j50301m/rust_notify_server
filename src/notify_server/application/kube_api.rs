use crate::config::config;
use k8s_openapi::api::core::v1::Pod;
use kgs_tracing::{tracing, warn};
use kube::{api::ListParams, Api, Client};

#[tracing::instrument]
pub async fn get_other_pod_ips() -> Vec<String> {
    let result: Result<Vec<String>, kube::Error> = async {
        // get k8s client
        let client = Client::try_default().await?;

        // get k8s config
        let kube_config = config::get_kubernetes();

        // use deployment name to get Pod list
        let pods: Api<Pod> = Api::namespaced(client, &kube_config.pod_namespace);

        let lp = ListParams::default().labels(&format!("app={}", kube_config.deployment_name));
        let pod_list = pods.list(&lp).await?;

        // get other pod ips
        let ip_vec: Vec<String> = pod_list
            .items
            .into_iter()
            .filter_map(|pod| {
                pod.status
                    .unwrap()
                    .pod_ip
                    .filter(|pod_ip| pod_ip != &kube_config.pod_ip)
            })
            .collect();

        Ok(ip_vec)
    }
    .await;

    match result {
        Ok(ip_vec) => ip_vec,
        Err(err) => {
            warn!("非k8s環境, 無法獲取其他Pod IP err:{}", err);
            vec![]
        }
    }
}

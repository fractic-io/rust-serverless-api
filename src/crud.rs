use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::http::Method;
use fractic_aws_dynamo::errors::DynamoNotFound;
use fractic_aws_dynamo::schema::{DynamoObject, PkSk};
use fractic_aws_dynamo::util::DynamoUtil;
use fractic_env_config::{load_env, EnvConfigEnum};
use fractic_server_error::common::CriticalError;
use fractic_server_error::GenericServerError;
use lambda_runtime::Error;
use lambda_runtime::LambdaEvent;

use crate::{build_error, build_result, parse_request_data, InvalidRequestError};

pub struct CrudRouteScaffolding {
    dynamo_util: DynamoUtil<aws_sdk_dynamodb::Client>,
}

#[derive(Debug)]
enum RequestProperties<T: DynamoObject> {
    Read { id: PkSk },
    Create { parent_id: PkSk, data: T::Data },
    Update { object: T },
    Delete { id: PkSk },
}

// Response data returned if an object was created:
#[derive(Debug, serde::Serialize)]
struct ObjectCreatedResponseData {
    created_id: PkSk,
}

impl CrudRouteScaffolding {
    pub async fn new<EnvConfig: EnvConfigEnum>(
        table_var: EnvConfig,
    ) -> Result<Self, GenericServerError> {
        let env = load_env::<EnvConfig>()?;
        let dynamo_util = DynamoUtil::new(env.clone_into()?, env.get(&table_var)?).await?;
        Ok(CrudRouteScaffolding { dynamo_util })
    }

    pub async fn handle_request<T: DynamoObject>(
        &self,
        event: LambdaEvent<ApiGatewayProxyRequest>,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        match Self::get_and_verify_request_properties::<T>(&event) {
            Ok(RequestProperties::<T>::Create { parent_id, data }) => {
                match self.create::<T>(parent_id, data).await {
                    Ok(result) => build_result(result),
                    Err(error) => build_error(error),
                }
            }
            Ok(RequestProperties::<T>::Read { id }) => match self.read::<T>(id).await {
                Ok(result) => build_result(result),
                Err(error) => build_error(error),
            },
            Ok(RequestProperties::<T>::Update { object }) => match self.update::<T>(object).await {
                Ok(result) => build_result(result),
                Err(error) => build_error(error),
            },
            Ok(RequestProperties::<T>::Delete { id }) => match self.delete::<T>(id).await {
                Ok(result) => build_result(result),
                Err(error) => build_error(error),
            },
            Err(e) => build_error(e),
        }
    }

    fn get_and_verify_request_properties<T: DynamoObject>(
        event: &LambdaEvent<ApiGatewayProxyRequest>,
    ) -> Result<RequestProperties<T>, GenericServerError> {
        let dbg_cxt: &'static str = "get_and_verify_request_properties";
        match &event.payload.http_method {
            &Method::POST => Ok(RequestProperties::<T>::Create {
                parent_id: PkSk::from_string(&Self::get_and_verify_query_param(
                    &event,
                    "parent_id",
                )?)?,
                data: parse_request_data::<T::Data>(&event.payload)?,
            }),
            &Method::GET => Ok(RequestProperties::<T>::Read {
                id: PkSk::from_string(&Self::get_and_verify_query_param(&event, "id")?)?,
            }),
            &Method::PUT => Ok(RequestProperties::<T>::Update {
                object: parse_request_data::<T>(&event.payload)?,
            }),
            &Method::DELETE => Ok(RequestProperties::<T>::Delete {
                id: PkSk::from_string(&Self::get_and_verify_query_param(&event, "id")?)?,
            }),
            _ => Err(CriticalError::new(
                dbg_cxt,
                "CRUD routes should only be called with POST, GET, PUT, or DELETE",
            )),
        }
    }

    fn get_and_verify_query_param(
        event: &LambdaEvent<ApiGatewayProxyRequest>,
        param: &str,
    ) -> Result<String, GenericServerError> {
        let dbg_cxt: &'static str = "get_and_verify_query_param";
        event
            .payload
            .query_string_parameters
            .first(param)
            .ok_or(InvalidRequestError::with_debug(
                dbg_cxt,
                "",
                format!("query parameter {} is required", param).to_string(),
            ))
            .map(|s| s.to_string())
    }

    async fn create<T: DynamoObject>(
        &self,
        parent_id: PkSk,
        data: T::Data,
    ) -> Result<ObjectCreatedResponseData, GenericServerError> {
        let written_obj = self
            .dynamo_util
            .create_item::<T>(parent_id, data, None)
            .await?;
        Ok(ObjectCreatedResponseData {
            created_id: written_obj.id().clone(),
        })
    }

    async fn read<T: DynamoObject>(&self, id: PkSk) -> Result<T, GenericServerError> {
        match self.dynamo_util.get_item(id).await? {
            Some(object) => Ok(object),
            None => Err(DynamoNotFound::default()),
        }
    }

    async fn update<T: DynamoObject>(&self, object: T) -> Result<(), GenericServerError> {
        self.dynamo_util.update_item(&object).await
    }

    async fn delete<T: DynamoObject>(&self, id: PkSk) -> Result<(), GenericServerError> {
        self.dynamo_util.delete_item::<T>(id).await
    }
}

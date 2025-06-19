use crate::middleware::extract_user_id_from_headers;
use crate::responses::ApiResponse;
use crate::services;
use crate::services::core::admin::SimpleAdminService;
use crate::types::UserAccessLevel;
use std::sync::Arc;
use worker::{console_log, Env, Request, Response, Result};

/// Get admin users statistics
pub async fn handle_api_admin_get_users(req: Request, env: Env) -> Result<Response> {
    console_log!("👑 Admin get users request");

    let user_id = match extract_user_id_from_headers(&req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Initialize services
    let kv_store = env.kv("ArbEdgeKV")?;
    let d1_database = Arc::new(env.d1("ArbEdgeD1")?);
    let encryption_key = env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    let d1_service = services::core::infrastructure::DatabaseManager::new(
        d1_database.clone(),
        services::core::infrastructure::database_repositories::DatabaseManagerConfig::default(),
    );

    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store.clone(),
        d1_service.clone(),
        encryption_key,
    );

    // Verify admin permissions
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(profile)) => {
            if profile.access_level != UserAccessLevel::Admin
                && profile.access_level != UserAccessLevel::SuperAdmin
            {
                let response = ApiResponse::<()>::error("Admin access required".to_string());
                return Ok(Response::from_json(&response)?.with_status(403));
            }

            // Initialize admin service
            let admin_service = SimpleAdminService::new(kv_store, d1_database);

            // Get user statistics
            match admin_service.get_user_statistics().await {
                Ok(stats) => {
                    let user_stats = serde_json::json!({
                        "total_users": stats.total_users,
                        "active_users": stats.active_users,
                        "users_by_type": {
                            "free": stats.free_users,
                            "paid": stats.paid_users,
                            "admin": stats.admin_users,
                            "super_admin": stats.super_admin_users,
                            "other": stats.other_users
                        },
                        "user_activity": {
                            "active_users": stats.active_users,
                            "recently_active_users": stats.recently_active_users,
                            "total_users": stats.total_users
                        },
                        "trading_activity": {
                            "total_trades": stats.total_trades,
                            "total_volume_usdt": stats.total_volume_usdt,
                            "generated_at": stats.generated_at
                        },
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    });

                    let response = ApiResponse::success(user_stats);
                    Response::from_json(&response)
                }
                Err(e) => {
                    let response =
                        ApiResponse::<()>::error(format!("Failed to fetch user statistics: {}", e));
                    Ok(Response::from_json(&response)?.with_status(500))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response =
                ApiResponse::<()>::error(format!("Failed to verify admin permissions: {}", e));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

/// Handle super admin system info request
pub async fn handle_api_admin_system_info(_req: Request, _env: Env) -> Result<Response> {
    console_log!("👑 Super admin system info request");

    let user_id = match extract_user_id_from_headers(&_req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Initialize services for admin verification
    let kv_store = _env.kv("ArbEdgeKV")?;
    let d1_database = Arc::new(_env.d1("ArbEdgeD1")?);
    let encryption_key = _env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    let d1_service = services::core::infrastructure::DatabaseManager::new(
        d1_database.clone(),
        services::core::infrastructure::database_repositories::DatabaseManagerConfig::default(),
    );

    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store.clone(),
        d1_service.clone(),
        encryption_key,
    );

    // Verify super admin permissions
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(profile)) => {
            if profile.access_level != UserAccessLevel::SuperAdmin {
                let response = ApiResponse::<()>::error("Super admin access required".to_string());
                return Ok(Response::from_json(&response)?.with_status(403));
            }

            // Initialize admin service
            let admin_service = SimpleAdminService::new(kv_store, d1_database);

            // Get comprehensive system info
            match admin_service.get_system_info().await {
                Ok(system_info) => {
                    let response = ApiResponse::success(system_info);
                    Response::from_json(&response)
                }
                Err(e) => {
                    let response =
                        ApiResponse::<()>::error(format!("Failed to fetch system info: {}", e));
                    Ok(Response::from_json(&response)?.with_status(500))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!(
                "Failed to verify super admin permissions: {}",
                e
            ));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

/// Handle super admin get config request
pub async fn handle_api_admin_get_config(_req: Request, _env: Env) -> Result<Response> {
    console_log!("👑 Super admin get config request");

    let user_id = match extract_user_id_from_headers(&_req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Initialize services for admin verification
    let kv_store = _env.kv("ArbEdgeKV")?;
    let d1_database = Arc::new(_env.d1("ArbEdgeD1")?);
    let encryption_key = _env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    let d1_service = services::core::infrastructure::DatabaseManager::new(
        d1_database.clone(),
        services::core::infrastructure::database_repositories::DatabaseManagerConfig::default(),
    );

    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store.clone(),
        d1_service.clone(),
        encryption_key,
    );

    // Verify super admin permissions
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(profile)) => {
            if profile.access_level != UserAccessLevel::SuperAdmin {
                let response = ApiResponse::<()>::error("Super admin access required".to_string());
                return Ok(Response::from_json(&response)?.with_status(403));
            }

            // Initialize admin service
            let admin_service = SimpleAdminService::new(kv_store, d1_database);

            // Get system configuration
            match admin_service.get_system_config().await {
                Ok(config) => {
                    let response = ApiResponse::success(config);
                    Response::from_json(&response)
                }
                Err(e) => {
                    let response =
                        ApiResponse::<()>::error(format!("Failed to fetch system config: {}", e));
                    Ok(Response::from_json(&response)?.with_status(500))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!(
                "Failed to verify super admin permissions: {}",
                e
            ));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

/// Handle super admin update config request
pub async fn handle_api_admin_update_config(_req: Request, _env: Env) -> Result<Response> {
    console_log!("👑 Super admin update config request");

    let user_id = match extract_user_id_from_headers(&_req) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::error("Authentication required".to_string());
            return Ok(Response::from_json(&response)?.with_status(401));
        }
    };

    // Initialize services for admin verification
    let kv_store = _env.kv("ArbEdgeKV")?;
    let d1_database = Arc::new(_env.d1("ArbEdgeD1")?);
    let encryption_key = _env
        .var("ENCRYPTION_KEY")
        .map(|secret| secret.to_string())
        .map_err(|_| {
            worker::Error::RustError("ENCRYPTION_KEY environment variable is required".to_string())
        })?;

    let d1_service = services::core::infrastructure::DatabaseManager::new(
        d1_database.clone(),
        services::core::infrastructure::database_repositories::DatabaseManagerConfig::default(),
    );

    let user_profile_service = services::core::user::user_profile::UserProfileService::new(
        kv_store.clone(),
        d1_service.clone(),
        encryption_key,
    );

    // Verify super admin permissions
    match user_profile_service.get_user_profile(&user_id).await {
        Ok(Some(profile)) => {
            if profile.access_level != UserAccessLevel::SuperAdmin {
                let response = ApiResponse::<()>::error("Super admin access required".to_string());
                return Ok(Response::from_json(&response)?.with_status(403));
            }

            // Parse update request
            let mut req = _req;
            let update_request: serde_json::Value = match req.json().await {
                Ok(data) => data,
                Err(e) => {
                    let response = ApiResponse::<()>::error(format!("Invalid JSON format: {}", e));
                    return Ok(Response::from_json(&response)?.with_status(400));
                }
            };

            // Initialize admin service
            let admin_service = SimpleAdminService::new(kv_store, d1_database);

            // Update system configuration
            match admin_service.update_system_config(update_request).await {
                Ok(updated_config) => {
                    let response = ApiResponse::success(serde_json::json!({
                        "updated": true,
                        "config": updated_config,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    }));
                    Response::from_json(&response)
                }
                Err(e) => {
                    let response =
                        ApiResponse::<()>::error(format!("Failed to update system config: {}", e));
                    Ok(Response::from_json(&response)?.with_status(500))
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<()>::error("User profile not found".to_string());
            Ok(Response::from_json(&response)?.with_status(404))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(format!(
                "Failed to verify super admin permissions: {}",
                e
            ));
            Ok(Response::from_json(&response)?.with_status(500))
        }
    }
}

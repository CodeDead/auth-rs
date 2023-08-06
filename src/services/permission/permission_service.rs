use crate::repository::audit::audit_model::Action::{Create, Delete, Read, Search, Update};
use crate::repository::audit::audit_model::ResourceType::Permission as PermissionResourceType;
use crate::repository::audit::audit_model::{Audit, ResourceIdType};
use crate::repository::permission::permission_model::Permission;
use crate::repository::permission::permission_repository::{Error, PermissionRepository};
use crate::services::audit::audit_service::AuditService;
use crate::services::role::role_service::RoleService;
use log::{error, info};
use mongodb::Database;

#[derive(Clone)]
pub struct PermissionService {
    pub permission_repository: PermissionRepository,
}

impl PermissionService {
    /// # Summary
    ///
    /// Create a new PermissionService.
    ///
    /// # Arguments
    ///
    /// * `permission_repository` - The PermissionRepository to be used by the PermissionService.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// ```
    ///
    /// # Returns
    ///
    /// * `PermissionService` - The new PermissionService.
    pub fn new(permission_repository: PermissionRepository) -> PermissionService {
        PermissionService {
            permission_repository,
        }
    }

    /// # Summary
    ///
    /// Create a new Permission entity.
    ///
    /// # Arguments
    ///
    /// * `new_permission` - The Permission entity to create.
    /// * `user_id` - The ID of the User creating the Permission entity.
    /// * `db` - The Database to create the Permission entity in.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    ///
    /// let permission = permission_service.find_by_name(String::from("name"), &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Option<Permission>` - The Permission entity.
    /// * `Error` - The Error that occurred.
    pub async fn create(
        &self,
        new_permission: Permission,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Permission, Error> {
        info!("Creating Permission: {}", new_permission);

        let new_audit = Audit::new(
            user_id,
            Create,
            &new_permission.id,
            ResourceIdType::PermissionId,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.create(new_permission, db).await
    }

    /// # Summary
    ///
    /// Find all Permission entities.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the User finding the Permission entities.
    /// * `db` - The Database to find the Permission entities in.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let user_id = String::from("user_id");
    /// let permissions = permission_service.find_all(user_id, &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Vec<Permission>` - The Permission entities.
    /// * `Error` - The Error that occurred.
    pub async fn find_all(
        &self,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Vec<Permission>, Error> {
        info!("Finding all permissions");

        let new_audit = Audit::new(
            user_id,
            Read,
            "",
            ResourceIdType::None,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.find_all(db).await
    }

    /// # Summary
    ///
    /// Find all Permission entities by id.
    ///
    /// # Arguments
    ///
    /// * `id_vec` - The Vector of IDs of the Permission entities.
    /// * `user_id` - The ID of the User finding the Permission entities.
    /// * `db` - The Database to find the Permission entities in.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let user_id = String::from("user_id");
    /// let id_vec = vec![String::from("id")];
    /// let permissions = permission_service.find_by_id_vec(user_id, id_vec, &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Vec<Permission>` - The Permission entities.
    /// * `Error` - The Error that occurred.
    pub async fn find_by_id_vec(
        &self,
        id_vec: Vec<String>,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Vec<Permission>, Error> {
        info!("Finding permissions by id_vec: {:?}", id_vec);

        let new_audit = Audit::new(
            user_id,
            Read,
            &format!("{:?}", id_vec),
            ResourceIdType::PermissionIdVec,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.find_by_id_vec(id_vec, db).await
    }

    /// # Summary
    ///
    /// Find a Permission entity by id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the Permission entity.
    /// * `db` - The Database to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    ///
    /// let permission = permission_service.find_by_id(String::from("id"), &db);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Option<Permission>` - The Permission entity.
    /// * `Error` - The Error that occurred.
    pub async fn find_by_id(
        &self,
        id: &str,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Option<Permission>, Error> {
        info!("Finding Permission by ID: {}", id);

        let new_audit = Audit::new(
            user_id,
            Read,
            id,
            ResourceIdType::PermissionId,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.find_by_id(id, db).await
    }

    /// # Summary
    ///
    /// Find a Permission by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the Permission to find.
    /// * `user_id` - The ID of the User finding the Permission.
    /// * `db` - The database to use.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let user_id = String::from("user_id");
    /// let name = String::from("name");
    /// let permission = permission_service.find_by_name(name, user_id, &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Result<Option<Permission>, Error>` - The result of the operation.
    pub async fn find_by_name(
        &self,
        name: &str,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Option<Permission>, Error> {
        info!("Finding Permission by name: {}", name);

        let new_audit = Audit::new(
            user_id,
            Read,
            name,
            ResourceIdType::PermissionName,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.find_by_name(name, db).await
    }

    /// # Summary
    ///
    /// Update a Permission entity.
    ///
    /// # Arguments
    ///
    /// * `permission` - The Permission entity to create.
    /// * `user_id` - The ID of the User updating the Permission.
    /// * `db` - The Database to be used.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let user_id = String::from("user_id");
    /// let permission = Permission::new(String::from("name"), String::from("description"));
    /// let updated_permission = permission_service.update(permission, user_id, &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Permission` - The Permission entity.
    /// * `Error` - The Error that occurred.
    pub async fn update(
        &self,
        permission: Permission,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Permission, Error> {
        info!("Updating Permission: {}", permission);

        let new_audit = Audit::new(
            user_id,
            Update,
            &permission.id,
            ResourceIdType::PermissionId,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.update(permission, db).await
    }

    /// # Summary
    ///
    /// Delete a Permission entity.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the Permission entity to delete.
    /// * `user_id` - The ID of the User deleting the Permission.
    /// * `db` - The Database to be used.
    /// * `role_service` - The RoleService to be used.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let role_service = RoleService::new(role_repository);
    /// let user_id = String::from("user_id");
    /// let id = String::from("id");
    /// permission_service.delete(id, user_id, &db, &role_service, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `()` - The operation was successful.
    /// * `Error` - The Error that occurred.
    pub async fn delete(
        &self,
        id: &str,
        user_id: &str,
        db: &Database,
        role_service: &RoleService,
        audit: &AuditService,
    ) -> Result<(), Error> {
        info!("Deleting Permission by ID: {}", id);

        let new_audit = Audit::new(
            user_id,
            Delete,
            &id,
            ResourceIdType::PermissionId,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository
            .delete(id, db, role_service)
            .await
    }

    /// # Summary
    ///
    /// Search for Permission entities by text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to search for.
    /// * `user_id` - The ID of the User searching for the Permission.
    /// * `db` - The Database to be used.
    /// * `audit` - The AuditService to be used.
    ///
    /// # Example
    ///
    /// ```
    /// let permission_repository = PermissionRepository::new(String::from("permissions"));
    /// let permission_service = PermissionService::new(permission_repository);
    /// let db = mongodb::Database::new();
    /// let audit_service = AuditService::new(audit_repository);
    /// let user_id = String::from("user_id");
    /// let text = String::from("text");
    /// permission_service.search(text, user_id, &db, &audit_service);
    /// ```
    ///
    /// # Returns
    ///
    /// * `Vec<Permission>` - The Permission entities.
    /// * `Error` - The Error that occurred.
    pub async fn search(
        &self,
        text: &str,
        user_id: &str,
        db: &Database,
        audit: &AuditService,
    ) -> Result<Vec<Permission>, Error> {
        info!("Searching for Permission by text: {}", text);

        let new_audit = Audit::new(
            user_id,
            Search,
            "",
            ResourceIdType::PermissionSearch,
            PermissionResourceType,
        );
        match audit.create(new_audit, db).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create Audit: {}", e);
                return Err(Error::Audit(e));
            }
        }

        self.permission_repository.search(text, db).await
    }
}

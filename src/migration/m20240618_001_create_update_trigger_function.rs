use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create trigger function
        self.create_trigger_function(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.delete_trigger_function(manager).await?;

        Ok(())
    }
}

impl Migration {
    async fn create_trigger_function(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            CREATE OR REPLACE FUNCTION update_timestamp()
            RETURNS TRIGGER AS $$
                BEGIN
                    IF NEW.update_at IS DISTINCT FROM OLD.update_at THEN
                        NEW.update_at = NOW();
                    END IF;
                    RETURN NEW;
                END;
            $$ LANGUAGE plpgsql;
        "#;
        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }

    async fn delete_trigger_function(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            DROP FUNCTION IF EXISTS update_timestamp();
        "#;
        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }
}

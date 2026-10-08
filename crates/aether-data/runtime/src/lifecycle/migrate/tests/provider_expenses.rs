use super::*;
use aether_data_contracts::repository::billing::{
    AdminBillingMutationOutcome, BillingReadRepository, ProviderExpenseInput, ProviderExpenseQuery,
};

#[tokio::test]
async fn postgres_provider_expense_ledger_migration_preserves_exact_sums_idempotency_and_voids() {
    let Some(server) = ManagedPostgresServer::try_start()
        .await
        .expect("local postgres should start or skip")
    else {
        return;
    };
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    // Exercise only this self-contained migration against an isolated database.
    sqlx::raw_sql(include_str!(
        "../../../../schema/bootstrap/postgres/210_provider_expenses.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let repository = crate::repository::billing::SqlxBillingReadRepository::new(pool.clone());
    let input = ProviderExpenseInput {
        client_request_id: uuid::Uuid::new_v4().to_string(),
        provider_id: "deleted-later".into(),
        provider_name: "Supplier".into(),
        kind: "recharge".into(),
        amount: "0.10000000".into(),
        currency: "USD".into(),
        paid_at_unix_ms: 1000,
        period_start_unix_ms: None,
        period_end_unix_ms: None,
        note: None,
        external_reference: None,
        created_by: Some("admin".into()),
    };
    let (a, b) = tokio::join!(
        repository.create_provider_expense(&input),
        repository.create_provider_expense(&input)
    );
    let (AdminBillingMutationOutcome::Applied(a), AdminBillingMutationOutcome::Applied(b)) =
        (a.unwrap(), b.unwrap())
    else {
        panic!("expected applied")
    };
    assert_eq!(a.id, b.id);
    let mut subscription = input.clone();
    subscription.client_request_id = uuid::Uuid::new_v4().to_string();
    subscription.amount = "0.20000000".into();
    subscription.kind = "subscription".into();
    let AdminBillingMutationOutcome::Applied(second) = repository
        .create_provider_expense(&subscription)
        .await
        .unwrap()
    else {
        panic!("expected applied")
    };
    let mut cny = input.clone();
    cny.client_request_id = uuid::Uuid::new_v4().to_string();
    cny.currency = "CNY".into();
    cny.amount = "7.00000000".into();
    repository.create_provider_expense(&cny).await.unwrap();
    let query = ProviderExpenseQuery {
        from_unix_ms: 0,
        to_unix_ms: 2000,
        limit: 1,
        offset: 1,
    };
    let page = repository
        .list_provider_expenses(&query)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.totals[0].currency, "CNY");
    assert_eq!(page.totals[1].amount, "0.30000000");
    assert_eq!(page.totals[1].subscription_amount, "0.20000000");
    let mut conflict = input.clone();
    conflict.amount = "5.00000000".into();
    assert!(matches!(
        repository.create_provider_expense(&conflict).await.unwrap(),
        AdminBillingMutationOutcome::Invalid(_)
    ));
    let AdminBillingMutationOutcome::Applied(voided) = repository
        .void_provider_expense(&second.id, Some("operator-1"))
        .await
        .unwrap()
    else {
        panic!("expected void")
    };
    let AdminBillingMutationOutcome::Applied(again) = repository
        .void_provider_expense(&second.id, Some("operator-2"))
        .await
        .unwrap()
    else {
        panic!("expected repeat void")
    };
    assert_eq!(voided, again);
    let page = repository
        .list_provider_expenses(&query)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.totals[1].amount, "0.10000000");
    let raw_count: i64 = sqlx::query_scalar("SELECT count(*) FROM provider_expenses")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(raw_count, 3);
    let invalid=sqlx::query("INSERT INTO provider_expenses(id,client_request_id,provider_id,provider_name,kind,amount,currency,paid_at) VALUES('bad','bad','p','P','recharge',-1,'USD',NOW())").execute(&pool).await;
    assert!(invalid.is_err());
    pool.close().await;
}

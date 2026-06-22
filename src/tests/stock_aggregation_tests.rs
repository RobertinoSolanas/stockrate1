// ============================================================
// Stock Aggregation Service Tests
// ============================================================

mod stock_aggregation_tests {
    use crate::providers::mock::MockDataProvider;
    use crate::providers::second_mock::SecondMockDataProvider;
    use crate::providers::StockDataProvider;
    use crate::services::stock_aggregation::*;

    fn make_providers() -> std::sync::Arc<std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>> {
        std::sync::Arc::new(std::sync::RwLock::new(vec![
            Box::new(MockDataProvider::new()),
            Box::new(SecondMockDataProvider::new()),
        ]))
    }

    fn make_service() -> StockAggregationService {
        StockAggregationService::new(make_providers())
    }

    // Service construction tests
    #[test]
    fn test_service_new() {
        let service = StockAggregationService::new(make_providers());
        assert!(service.providers.read().unwrap().len() >= 2);
    }

    // MetricType tests
    #[test]
    fn test_metric_type_label_pe() {
        assert_eq!(MetricType::PE.label(), "P/E Ratio");
    }

    #[test]
    fn test_metric_type_label_roe() {
        assert_eq!(MetricType::ROE.label(), "ROE");
    }

    #[test]
    fn test_metric_type_label_revenue_growth() {
        assert_eq!(MetricType::RevenueGrowth3Y.label(), "Revenue Growth 3Y");
    }

    #[test]
    fn test_metric_type_unit_roe() {
        assert_eq!(MetricType::ROE.unit(), "%");
    }

    #[test]
    fn test_metric_type_unit_fcf() {
        assert_eq!(MetricType::FCF.unit(), "$");
    }

    #[test]
    fn test_metric_type_unit_pe() {
        assert_eq!(MetricType::PE.unit(), "");
    }

    #[test]
    fn test_metric_type_color() {
        assert_eq!(MetricType::PE.color(), "#3b82f6");
        assert_eq!(MetricType::ROE.color(), "#10b981");
        assert_eq!(MetricType::Upside.color(), "#8b5cf6");
    }

    #[test]
    fn test_metric_type_from_str_pe() {
        assert!(MetricType::from_str("pe").is_some());
        assert!(MetricType::from_str("pe_ratio").is_some());
    }

    #[test]
    fn test_metric_type_from_str_roe() {
        assert!(MetricType::from_str("roe").is_some());
        assert!(MetricType::from_str("return_on_equity").is_some());
    }

    #[test]
    fn test_metric_type_from_str_revenue_growth() {
        assert!(MetricType::from_str("revenue").is_some());
        assert!(MetricType::from_str("revenue_growth").is_some());
        assert!(MetricType::from_str("rev_growth").is_some());
    }

    #[test]
    fn test_metric_type_from_str_upside() {
        assert!(MetricType::from_str("upside").is_some());
        assert!(MetricType::from_str("upside_pct").is_some());
    }

    #[test]
    fn test_metric_type_from_str_unknown() {
        assert!(MetricType::from_str("unknown_metric").is_none());
    }

    #[test]
    fn test_metric_type_from_str_case_insensitive() {
        assert!(MetricType::from_str("PE").is_some());
        assert!(MetricType::from_str("Roe").is_some());
        assert!(MetricType::from_str("UPSIDE").is_some());
    }

    // ChartType tests
    #[test]
    fn test_chart_type_from_str_bar() {
        assert_eq!(ChartType::from_str("bar"), Some(ChartType::Bar));
    }

    #[test]
    fn test_chart_type_from_str_horizontal() {
        assert_eq!(ChartType::from_str("horizontal"), Some(ChartType::HorizontalBar));
        assert_eq!(ChartType::from_str("hbar"), Some(ChartType::HorizontalBar));
    }

    #[test]
    fn test_chart_type_from_str_scatter() {
        assert_eq!(ChartType::from_str("scatter"), Some(ChartType::Scatter));
    }

    #[test]
    fn test_chart_type_from_str_unknown() {
        assert!(ChartType::from_str("invalid").is_none());
    }

    // Default config tests
    #[test]
    fn test_default_config() {
        let config = StockAggregationService::default_config();
        assert_eq!(config.metrics.len(), 4);
        assert!(config.metrics.contains(&MetricType::PE));
        assert!(config.metrics.contains(&MetricType::ROE));
        assert!(config.metrics.contains(&MetricType::RevenueGrowth3Y));
        assert!(config.metrics.contains(&MetricType::Upside));
    }

    // Parse query params tests
    #[test]
    fn test_parse_query_params_empty() {
        let params: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let config = StockAggregationService::parse_query_params(&params);
        assert_eq!(config.metrics, StockAggregationService::default_config().metrics);
    }

    #[test]
    fn test_parse_query_params_metrics() {
        let mut params = std::collections::HashMap::new();
        params.insert("metrics".to_string(), "pe,roe".to_string());
        let config = StockAggregationService::parse_query_params(&params);
        assert_eq!(config.metrics.len(), 2);
        assert!(config.metrics.contains(&MetricType::PE));
        assert!(config.metrics.contains(&MetricType::ROE));
    }

    #[test]
    fn test_parse_query_params_invalid_metric_ignored() {
        let mut params = std::collections::HashMap::new();
        params.insert("metrics".to_string(), "pe,invalid,roe".to_string());
        let config = StockAggregationService::parse_query_params(&params);
        assert_eq!(config.metrics.len(), 2);
        assert!(config.metrics.contains(&MetricType::PE));
        assert!(config.metrics.contains(&MetricType::ROE));
    }

    #[test]
    fn test_parse_query_params_chart_type() {
        let mut params = std::collections::HashMap::new();
        params.insert("chart_type".to_string(), "scatter".to_string());
        let config = StockAggregationService::parse_query_params(&params);
        assert_eq!(config.chart_type, ChartType::Scatter);
    }

    // Get all tickers tests
    #[test]
    fn test_get_all_tickers() {
        let service = make_service();
        let tickers = service.get_all_tickers();
        assert!(!tickers.is_empty());

        // AAPL should be available from both providers
        let aapl = tickers.iter().find(|t| t.ticker == "AAPL");
        assert!(aapl.is_some());
        let aapl = aapl.unwrap();
        assert_eq!(aapl.providers.len(), 2);
    }

    #[test]
    fn test_get_all_tickers_sorted() {
        let service = make_service();
        let tickers = service.get_all_tickers();
        let tickers_sorted: Vec<String> = tickers.iter().map(|t| t.ticker.clone()).collect();
        let mut sorted = tickers_sorted.clone();
        sorted.sort();
        assert_eq!(tickers_sorted, sorted);
    }

    // Get aggregated data tests
    #[test]
    fn test_get_aggregated_data_returns_data() {
        let service = make_service();
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);
        assert!(!data.tickers.is_empty());
        assert!(!data.chart_groups.is_empty());
    }

    #[test]
    fn test_get_aggregated_data_includes_tickers() {
        let service = make_service();
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);

        assert!(data.tickers.contains(&"AAPL".to_string()));
        assert!(data.tickers.contains(&"MSFT".to_string()));
        assert!(data.tickers.contains(&"GOOGL".to_string()));
        assert!(data.tickers.contains(&"TSLA".to_string()));
    }

    #[test]
    fn test_get_aggregated_data_chart_groups_have_entries() {
        let service = make_service();
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);

        for group in &data.chart_groups {
            assert!(!group.entries.is_empty(), "Chart group '{}' should have entries", group.label);
            for entry in &group.entries {
                assert!(!entry.ticker.is_empty());
            }
        }
    }

    #[test]
    fn test_get_aggregated_data_pe_metric_values() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![MetricType::PE];
        let data = service.get_aggregated_data(&config);

        assert_eq!(data.chart_groups.len(), 1);
        let group = &data.chart_groups[0];
        assert_eq!(group.label, "P/E Ratio");

        // Verify that entries have actual P/E values
        for entry in &group.entries {
            // P/E ratios should be positive for real stocks
            assert!(entry.value >= 0.0);
        }
    }

    #[test]
    fn test_get_aggregated_data_roe_metric_values() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![MetricType::ROE];
        let data = service.get_aggregated_data(&config);

        assert_eq!(data.chart_groups.len(), 1);
        let group = &data.chart_groups[0];
        assert_eq!(group.label, "ROE");
        assert!(!group.entries.is_empty());
    }

    #[test]
    fn test_get_aggregated_data_provider_filter() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.provider_filter = vec!["MockDataProvider".to_string()];
        let data = service.get_aggregated_data(&config);

        for group in &data.chart_groups {
            for entry in &group.entries {
                assert_eq!(entry.provider, "MockDataProvider");
            }
        }
        assert_eq!(data.providers, vec!["MockDataProvider"]);
    }

    #[test]
    fn test_get_aggregated_data_second_provider_filter() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.provider_filter = vec!["SecondMockDataProvider".to_string()];
        let data = service.get_aggregated_data(&config);

        for group in &data.chart_groups {
            for entry in &group.entries {
                assert_eq!(entry.provider, "SecondMockDataProvider");
            }
        }
        assert_eq!(data.providers, vec!["SecondMockDataProvider"]);
    }

    #[test]
    fn test_extract_metric_value_pe() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::PE, &data);
        assert_eq!(value, 28.5);
    }

    #[test]
    fn test_extract_metric_value_roe() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::ROE, &data);
        // ROE is stored as 1.54, should be returned as 154.0%
        assert!((value - 154.0).abs() < 0.01);
    }

    #[test]
    fn test_extract_metric_value_upside() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::Upside, &data);
        // Target: 210.50, Current: 185.25 -> (210.50-185.25)/185.25 * 100 = 13.63%
        assert!((value - 13.63).abs() < 0.1);
    }

    #[test]
    fn test_extract_metric_value_target_price() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::TargetPrice, &data);
        assert_eq!(value, 210.50);
    }

    #[test]
    fn test_extract_metric_value_current_price() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::CurrentPrice, &data);
        assert_eq!(value, 185.25);
    }

    #[test]
    fn test_extract_metric_value_analyst_count() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::AnalystCount, &data);
        assert_eq!(value, 42.0);
    }

    #[test]
    fn test_extract_metric_value_revenue_growth() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::RevenueGrowth3Y, &data);
        // Revenue growth is stored as 0.085, should be returned as 8.5%
        assert!((value - 8.5).abs() < 0.01);
    }

    #[test]
    fn test_extract_metric_value_eps_growth() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::EPSGrowth3Y, &data);
        // EPS growth is stored as 0.112, should be returned as 11.2%
        assert!((value - 11.2).abs() < 0.01);
    }

    #[test]
    fn test_extract_metric_value_debt_to_equity() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::DebtToEquity, &data);
        assert_eq!(value, 1.45);
    }

    #[test]
    fn test_extract_metric_value_current_ratio() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::CurrentRatio, &data);
        assert_eq!(value, 0.98);
    }

    #[test]
    fn test_extract_metric_value_forward_pe() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::ForwardPE, &data);
        assert_eq!(value, 25.2);
    }

    #[test]
    fn test_extract_metric_value_ev_to_ebitda() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::EVToEBITDA, &data);
        assert_eq!(value, 18.3);
    }

    #[test]
    fn test_extract_metric_value_pb() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::PB, &data);
        assert_eq!(value, 35.1);
    }

    #[test]
    fn test_extract_metric_value_fcf() {
        let mock = MockDataProvider::new();
        let data = mock.get_stock_data("AAPL").unwrap();
        let value = StockAggregationService::extract_metric_value(&MetricType::FCF, &data);
        assert_eq!(value, 97500000000.0);
    }

    // Provider filter tests
    #[test]
    fn test_should_include_provider_empty_filter_includes_all() {
        assert!(StockAggregationService::should_include_provider(&[], "MockDataProvider"));
        assert!(StockAggregationService::should_include_provider(&[], "AnyProvider"));
    }

    #[test]
    fn test_should_include_provider_matching_filter() {
        assert!(StockAggregationService::should_include_provider(&["MockDataProvider".to_string()], "MockDataProvider"));
    }

    #[test]
    fn test_should_include_provider_non_matching_filter() {
        assert!(!StockAggregationService::should_include_provider(&["SecondMockDataProvider".to_string()], "MockDataProvider"));
    }

    #[test]
    fn test_should_include_provider_case_insensitive() {
        assert!(StockAggregationService::should_include_provider(&["mockdatapROVIDER".to_string()], "MockDataProvider"));
    }

    // All providers test
    #[test]
    fn test_get_aggregated_data_all_providers() {
        let service = make_service();
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);

        assert!(data.providers.contains(&"MockDataProvider".to_string()));
        assert!(data.providers.contains(&"SecondMockDataProvider".to_string()));
    }

    // Multiple metrics test
    #[test]
    fn test_get_aggregated_data_multiple_metrics() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![
            MetricType::PE,
            MetricType::ROE,
            MetricType::RevenueGrowth3Y,
            MetricType::Upside,
        ];
        let data = service.get_aggregated_data(&config);

        // Should have exactly 4 chart groups
        assert_eq!(data.chart_groups.len(), 4);
    }

    // Second provider specific tests
    #[test]
    fn test_second_provider_nvda_data() {
        let second = SecondMockDataProvider::new();
        let service = StockAggregationService::new(std::sync::Arc::new(std::sync::RwLock::new(vec![
            Box::new(second),
        ])));
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![MetricType::PE];
        let data = service.get_aggregated_data(&config);

        let group = &data.chart_groups[0];
        let nvda_entry = group.entries.iter().find(|e| e.ticker == "NVDA");
        assert!(nvda_entry.is_some());
        let nvda = nvda_entry.unwrap();
        assert_eq!(nvda.value, 65.8);
    }

    #[test]
    fn test_second_provider_meta_data() {
        let second = SecondMockDataProvider::new();
        let service = StockAggregationService::new(std::sync::Arc::new(std::sync::RwLock::new(vec![
            Box::new(second),
        ])));
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![MetricType::PE];
        let data = service.get_aggregated_data(&config);

        let group = &data.chart_groups[0];
        let meta_entry = group.entries.iter().find(|e| e.ticker == "META");
        assert!(meta_entry.is_some());
    }

    // Cross-provider AAPL comparison
    #[test]
    fn test_cross_provider_aapl_pe_different() {
        let service = make_service();
        let mut config = StockAggregationService::default_config();
        config.metrics = vec![MetricType::PE];
        let data = service.get_aggregated_data(&config);

        let group = &data.chart_groups[0];
        let aapl_entries: Vec<_> = group.entries.iter().filter(|e| e.ticker == "AAPL").collect();
        assert_eq!(aapl_entries.len(), 2);

        let values: Vec<f64> = aapl_entries.iter().map(|e| e.value).collect();
        assert!(values[0] != values[1], "Different providers should return different P/E for AAPL");
    }

    // All stocks data test
    #[test]
    fn test_get_all_stock_data_returns_data() {
        let service = make_service();
        let data = service.get_all_stock_data();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_get_all_stock_data_has_providers() {
        let service = make_service();
        let data = service.get_all_stock_data();
        let providers: Vec<_> = data.iter().map(|(_, provider, _)| provider.as_str()).collect();
        assert!(providers.contains(&"MockDataProvider"));
        assert!(providers.contains(&"SecondMockDataProvider"));
    }
}

// ============================================================
// Routes Handler Tests
// ============================================================

mod routes_handler_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use crate::providers::mock::MockDataProvider;
    use crate::providers::second_mock::SecondMockDataProvider;
    use crate::providers::StockDataProvider;
    use crate::routes::*;
    use crate::services::stock_aggregation::*;

    fn make_providers() -> Vec<Box<dyn StockDataProvider + Send + Sync>> {
        vec![
            Box::new(MockDataProvider::new()),
            Box::new(SecondMockDataProvider::new()),
        ]
    }

    fn make_router() -> axum::Router {
        let state = AppState {
            providers: std::sync::Arc::new(std::sync::RwLock::new(make_providers())),
        };

        Router::new()
            .route("/", get(index_handler))
            .route("/api/all-stocks", get(all_stocks_handler))
            .route("/api/all-tickers", get(all_tickers_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_index_handler_returns_200() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_index_handler_returns_html() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Market Overview"));
        assert!(body_str.contains("Multi-Provider"));
    }

    #[tokio::test]
    async fn test_index_handler_contains_charts() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("config-bar"));
        assert!(body_str.contains("charts-container"));
        assert!(body_str.contains("chart-section-card"));
    }

    #[tokio::test]
    async fn test_api_all_stocks_returns_200() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-stocks").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_all_stocks_returns_json() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-stocks").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("tickers").is_some());
        assert!(json.get("chart_groups").is_some());
        assert!(json.get("providers").is_some());
        assert!(json.get("metrics").is_some());

        let tickers = json["tickers"].as_array().unwrap();
        assert!(!tickers.is_empty());

        let chart_groups = json["chart_groups"].as_array().unwrap();
        assert!(!chart_groups.is_empty());
    }

    #[tokio::test]
    async fn test_api_all_stocks_contains_aapl() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-stocks").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tickers = json["tickers"].as_array().unwrap();
        let ticker_names: Vec<&str> = tickers.iter().filter_map(|t| t.as_str()).collect();
        assert!(ticker_names.contains(&"AAPL"));
    }

    #[tokio::test]
    async fn test_api_all_stocks_has_chart_groups_with_entries() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-stocks").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let chart_groups = json["chart_groups"].as_array().unwrap();
        for group in chart_groups {
            assert!(group.get("entries").is_some(), "Chart group should have entries");
            let entries = group["entries"].as_array().unwrap();
            assert!(!entries.is_empty(), "Chart group '{}' should not be empty", group["metric"]);
        }
    }

    #[tokio::test]
    async fn test_api_all_tickers_returns_200() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-tickers").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_all_tickers_returns_json() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-tickers").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("tickers").is_some());
        assert!(json.get("total").is_some());

        let total = json["total"].as_u64().unwrap();
        assert!(total > 0);
    }

    #[tokio::test]
    async fn test_api_all_tickers_contains_aapl() {
        let router = make_router();
        let response = router
            .oneshot(Request::builder().uri("/api/all-tickers").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tickers = json["tickers"].as_array().unwrap();
        let aapl = tickers.iter().find(|t| t["ticker"] == "AAPL");
        assert!(aapl.is_some());
        let aapl = aapl.unwrap();
        let providers = aapl["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 2);
    }

    #[tokio::test]
    async fn test_html_index_contains_configurable_chart() {
        let service = StockAggregationService::new(std::sync::Arc::new(std::sync::RwLock::new(make_providers())));
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);

        let tickers: Vec<String> = vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(), "TSLA".to_string(),
        ];

        let html = html_index(&data.chart_groups, &tickers);
        assert!(html.contains("Market Overview"));
        assert!(html.contains("Multi-Provider Aggregation"));
        assert!(html.contains("config-bar"));
        assert!(html.contains("charts-container"));
        assert!(html.contains("chart-section-card"));
    }

    #[tokio::test]
    async fn test_html_index_contains_metric_rows() {
        let service = StockAggregationService::new(std::sync::Arc::new(std::sync::RwLock::new(make_providers())));
        let config = StockAggregationService::default_config();
        let data = service.get_aggregated_data(&config);

        let tickers: Vec<String> = vec![
            "AAPL".to_string(), "MSFT".to_string(),
        ];

        let html = html_index(&data.chart_groups, &tickers);
        assert!(html.contains("P/E Ratio"));
        assert!(html.contains("ROE"));
        assert!(html.contains("Revenue Growth 3Y"));
    }
}

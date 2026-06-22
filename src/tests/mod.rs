// ============================================================
// Finnhub Provider Tests
// ============================================================

mod finnhub_tests;

// ============================================================
// Stock Aggregation Service Tests
// ============================================================

mod stock_aggregation_tests;

// ============================================================
// Models Tests
// ============================================================

mod models_tests {
    use crate::models::*;

    #[test]
    fn test_recommendation_serialize_strong_buy() {
        let rec = Recommendation::StrongBuy;
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(json, "\"STRONG_BUY\"");
    }

    #[test]
    fn test_recommendation_serialize_buy() {
        let rec = Recommendation::Buy;
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(json, "\"BUY\"");
    }

    #[test]
    fn test_recommendation_serialize_hold() {
        let rec = Recommendation::Hold;
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(json, "\"HOLD\"");
    }

    #[test]
    fn test_recommendation_serialize_sell() {
        let rec = Recommendation::Sell;
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(json, "\"SELL\"");
    }

    #[test]
    fn test_recommendation_serialize_strong_sell() {
        let rec = Recommendation::StrongSell;
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(json, "\"STRONG_SELL\"");
    }

    #[test]
    fn test_recommendation_deserialize_strong_buy() {
        let rec: Recommendation = serde_json::from_str("\"STRONG_BUY\"").unwrap();
        assert!(matches!(rec, Recommendation::StrongBuy));
    }

    #[test]
    fn test_recommendation_deserialize_buy() {
        let rec: Recommendation = serde_json::from_str("\"BUY\"").unwrap();
        assert!(matches!(rec, Recommendation::Buy));
    }

    #[test]
    fn test_recommendation_deserialize_hold() {
        let rec: Recommendation = serde_json::from_str("\"HOLD\"").unwrap();
        assert!(matches!(rec, Recommendation::Hold));
    }

    #[test]
    fn test_valuation_ratios_default() {
        let v = ValuationRatios::default();
        assert!(v.pe_ratio.is_none());
        assert!(v.forward_pe_ratio.is_none());
        assert!(v.ev_to_ebitda.is_none());
        assert!(v.pb_ratio.is_none());
    }

    #[test]
    fn test_valuation_ratios_serialize_with_values() {
        let v = ValuationRatios {
            pe_ratio: Some(25.0),
            forward_pe_ratio: Some(20.0),
            ev_to_ebitda: Some(15.0),
            pb_ratio: Some(5.0),
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("\"pe_ratio\":25.0"));
        assert!(json.contains("\"forward_pe_ratio\":20.0"));
        assert!(json.contains("\"ev_to_ebitda\":15.0"));
        assert!(json.contains("\"pb_ratio\":5.0"));
    }

    #[test]
    fn test_valuation_ratios_skip_serializing_none() {
        let v = ValuationRatios {
            pe_ratio: Some(25.0),
            forward_pe_ratio: None,
            ev_to_ebitda: None,
            pb_ratio: None,
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("\"pe_ratio\":25.0"));
        assert!(!json.contains("forward_pe_ratio"));
        assert!(!json.contains("pb_ratio"));
        assert!(json.contains("ev_to_ebitda"));
    }

    #[test]
    fn test_financial_health_default() {
        let f = FinancialHealth::default();
        assert!(f.return_on_equity.is_none());
        assert!(f.debt_to_equity.is_none());
        assert!(f.free_cash_flow.is_none());
        assert!(f.current_ratio.is_none());
    }

    #[test]
    fn test_growth_metrics_default() {
        let g = GrowthMetrics::default();
        assert!(g.revenue_growth_3y.is_none());
        assert!(g.eps_growth_3y.is_none());
    }

    #[test]
    fn test_market_sentiment_default() {
        let m = MarketSentiment::default();
        assert!(m.target_price_consensus.is_none());
        assert!(m.current_price.is_none());
        assert!(m.recommendation_consensus.is_none());
        assert!(m.analyst_count.is_none());
    }

    #[test]
    fn test_stock_rating_data_clone() {
        let data = StockRatingData {
            ticker: "AAPL".to_string(),
            company_name: "Apple Inc.".to_string(),
            provider: "Test".to_string(),
            last_updated: None,
            valuation_ratios: ValuationRatios::default(),
            financial_health: FinancialHealth::default(),
            growth_metrics: GrowthMetrics::default(),
            market_sentiment: MarketSentiment::default(),
        };
        let cloned = data.clone();
        assert_eq!(data.ticker, cloned.ticker);
        assert_eq!(data.company_name, cloned.company_name);
    }

    #[test]
    fn test_stock_rating_data_serialize() {
        let data = StockRatingData {
            ticker: "AAPL".to_string(),
            company_name: "Apple Inc.".to_string(),
            provider: "Mock".to_string(),
            last_updated: None,
            valuation_ratios: ValuationRatios::default(),
            financial_health: FinancialHealth::default(),
            growth_metrics: GrowthMetrics::default(),
            market_sentiment: MarketSentiment::default(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"ticker\":\"AAPL\""));
        assert!(json.contains("\"company_name\":\"Apple Inc.\""));
        assert!(json.contains("\"provider\":\"Mock\""));
    }

    #[test]
    fn test_stock_rating_data_deserialize() {
        let json = r#"{"ticker":"TSLA","company_name":"Tesla Inc.","provider":"Test","valuation_ratios":{"pe_ratio":85.0},"financial_health":{"return_on_equity":0.25},"growth_metrics":{"eps_growth_3y":0.42},"market_sentiment":{"target_price_consensus":280.0,"current_price":245.0,"recommendation_consensus":"HOLD","analyst_count":35}}"#;
        let data: StockRatingData = serde_json::from_str(json).unwrap();
        assert_eq!(data.ticker, "TSLA");
        assert_eq!(data.company_name, "Tesla Inc.");
        assert_eq!(data.valuation_ratios.pe_ratio, Some(85.0));
        assert_eq!(data.financial_health.return_on_equity, Some(0.25));
        assert_eq!(data.growth_metrics.eps_growth_3y, Some(0.42));
        assert_eq!(data.market_sentiment.target_price_consensus, Some(280.0));
        assert_eq!(data.market_sentiment.current_price, Some(245.0));
        assert_eq!(data.market_sentiment.analyst_count, Some(35));
    }
}

// ============================================================
// MockDataProvider Tests
// ============================================================

mod mock_data_provider_tests {
    use crate::providers::mock::MockDataProvider;
    use crate::providers::StockDataProvider;

    #[test]
    fn test_mock_data_provider_new() {
        let _provider = MockDataProvider::new();
    }

    #[test]
    fn test_mock_data_provider_returns_aapl() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("AAPL");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "AAPL");
        assert_eq!(data.company_name, "Apple Inc.");
        assert_eq!(data.provider, "MockDataProvider");
    }

    #[test]
    fn test_mock_data_provider_returns_msft() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("MSFT");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "MSFT");
        assert_eq!(data.company_name, "Microsoft Corporation");
    }

    #[test]
    fn test_mock_data_provider_returns_googl() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("GOOGL");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "GOOGL");
        assert_eq!(data.company_name, "Alphabet Inc.");
    }

    #[test]
    fn test_mock_data_provider_returns_tsla() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("TSLA");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "TSLA");
        assert_eq!(data.company_name, "Tesla Inc.");
    }

    #[test]
    fn test_mock_data_provider_returns_amzn() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("AMZN");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "AMZN");
        assert_eq!(data.company_name, "Amazon.com Inc.");
    }

    #[test]
    fn test_mock_data_provider_case_insensitive() {
        let provider = MockDataProvider::new();
        let data_lower = provider.get_stock_data("aapl");
        let data_upper = provider.get_stock_data("AAPL");
        assert!(data_lower.is_some());
        assert!(data_upper.is_some());
        assert_eq!(data_lower.unwrap().ticker, "AAPL");
        assert_eq!(data_upper.unwrap().ticker, "AAPL");
    }

    #[test]
    fn test_mock_data_provider_returns_none_for_unknown() {
        let provider = MockDataProvider::new();
        assert!(provider.get_stock_data("UNKNOWN").is_none());
        assert!(provider.get_stock_data("XYZ").is_none());
    }

    #[test]
    fn test_mock_data_provider_list_supported_tickers() {
        let provider = MockDataProvider::new();
        let tickers = provider.list_supported_tickers();
        assert_eq!(tickers.len(), 5);
        assert!(tickers.contains(&"AAPL".to_string()));
        assert!(tickers.contains(&"MSFT".to_string()));
        assert!(tickers.contains(&"GOOGL".to_string()));
        assert!(tickers.contains(&"TSLA".to_string()));
        assert!(tickers.contains(&"AMZN".to_string()));
    }

    #[test]
    fn test_mock_data_provider_name() {
        let provider = MockDataProvider::new();
        assert_eq!(provider.provider_name(), "MockDataProvider");
    }

    #[test]
    fn test_mock_aapl_has_complete_data() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("AAPL").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(28.5));
        assert_eq!(data.valuation_ratios.forward_pe_ratio, Some(25.2));
        assert_eq!(data.valuation_ratios.ev_to_ebitda, Some(18.3));
        assert_eq!(data.valuation_ratios.pb_ratio, Some(35.1));
        assert_eq!(data.financial_health.return_on_equity, Some(1.54));
        assert_eq!(data.financial_health.debt_to_equity, Some(1.45));
        assert_eq!(data.financial_health.free_cash_flow, Some(97500000000));
        assert_eq!(data.financial_health.current_ratio, Some(0.98));
        assert_eq!(data.growth_metrics.revenue_growth_3y, Some(0.085));
        assert_eq!(data.growth_metrics.eps_growth_3y, Some(0.112));
        assert_eq!(data.market_sentiment.target_price_consensus, Some(210.50));
        assert_eq!(data.market_sentiment.current_price, Some(185.25));
        assert_eq!(data.market_sentiment.analyst_count, Some(42));
    }

    #[test]
    fn test_mock_msft_has_complete_data() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("MSFT").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(35.2));
        assert_eq!(data.financial_health.return_on_equity, Some(0.42));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::StrongBuy));
        assert_eq!(data.market_sentiment.target_price_consensus, Some(520.00));
    }

    #[test]
    fn test_mock_tsla_has_complete_data() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("TSLA").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(85.3));
        assert_eq!(data.growth_metrics.revenue_growth_3y, Some(0.350));
        assert_eq!(data.growth_metrics.eps_growth_3y, Some(0.420));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::Hold));
    }

    #[test]
    fn test_mock_amzn_has_complete_data() {
        let provider = MockDataProvider::new();
        let data = provider.get_stock_data("AMZN").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(52.5));
        assert_eq!(data.growth_metrics.revenue_growth_3y, Some(0.145));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::Buy));
    }
}

// ============================================================
// SecondMockDataProvider Tests
// ============================================================

mod second_mock_data_provider_tests {
    use crate::providers::second_mock::SecondMockDataProvider;
    use crate::providers::StockDataProvider;

    #[test]
    fn test_second_provider_new() {
        let _provider = SecondMockDataProvider::new();
    }

    #[test]
    fn test_second_provider_returns_aapl() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("AAPL");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "AAPL");
        assert_eq!(data.provider, "SecondMockProvider");
    }

    #[test]
    fn test_second_provider_returns_msft() {
        let provider = SecondMockDataProvider::new();
        assert!(provider.get_stock_data("MSFT").is_some());
    }

    #[test]
    fn test_second_provider_returns_nvda() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("NVDA");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "NVDA");
        assert_eq!(data.company_name, "NVIDIA Corporation");
    }

    #[test]
    fn test_second_provider_returns_meta() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("META");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "META");
        assert_eq!(data.company_name, "Meta Platforms Inc.");
    }

    #[test]
    fn test_second_provider_returns_amd() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("AMD");
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.ticker, "AMD");
        assert_eq!(data.company_name, "Advanced Micro Devices Inc.");
    }

    #[test]
    fn test_second_provider_returns_none_for_unknown() {
        let provider = SecondMockDataProvider::new();
        assert!(provider.get_stock_data("UNKNOWN").is_none());
        assert!(provider.get_stock_data("AAPL2").is_none());
    }

    #[test]
    fn test_second_provider_list_supported_tickers() {
        let provider = SecondMockDataProvider::new();
        let tickers = provider.list_supported_tickers();
        assert_eq!(tickers.len(), 5);
        assert!(tickers.contains(&"AAPL".to_string()));
        assert!(tickers.contains(&"MSFT".to_string()));
        assert!(tickers.contains(&"NVDA".to_string()));
        assert!(tickers.contains(&"META".to_string()));
        assert!(tickers.contains(&"AMD".to_string()));
    }

    #[test]
    fn test_second_provider_name() {
        let provider: Box<dyn StockDataProvider + Send + Sync> = Box::new(SecondMockDataProvider::new());
        assert_eq!(provider.provider_name(), "SecondMockDataProvider");
        assert_eq!(provider.list_supported_tickers().len(), 5);
        let data = provider.get_stock_data("AAPL").unwrap();
        assert_eq!(data.ticker, "AAPL");
        assert_eq!(data.provider, "SecondMockProvider");
    }

    #[test]
    fn test_second_nvda_has_complete_data() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("NVDA").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(65.8));
        assert_eq!(data.growth_metrics.revenue_growth_3y, Some(0.520));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::StrongBuy));
        assert_eq!(data.market_sentiment.target_price_consensus, Some(165.00));
    }

    #[test]
    fn test_second_meta_has_complete_data() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("META").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(26.5));
        assert_eq!(data.financial_health.debt_to_equity, Some(0.05));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::Buy));
    }

    #[test]
    fn test_second_amd_has_complete_data() {
        let provider = SecondMockDataProvider::new();
        let data = provider.get_stock_data("AMD").unwrap();
        assert_eq!(data.valuation_ratios.pe_ratio, Some(58.2));
        assert_eq!(data.growth_metrics.revenue_growth_3y, Some(0.185));
        assert_eq!(data.market_sentiment.recommendation_consensus, Some(crate::models::Recommendation::Hold));
    }

    #[test]
    fn test_different_providers_return_different_aapl_data() {
        let mock = crate::providers::mock::MockDataProvider::new();
        let second = SecondMockDataProvider::new();
        let aapl1 = mock.get_stock_data("AAPL").unwrap();
        let aapl2 = second.get_stock_data("AAPL").unwrap();
        assert_eq!(aapl1.ticker, aapl2.ticker);
        assert_eq!(aapl1.company_name, aapl2.company_name);
        assert_ne!(aapl1.provider, aapl2.provider);
        assert_ne!(aapl1.valuation_ratios.pe_ratio, aapl2.valuation_ratios.pe_ratio);
        assert_ne!(aapl1.financial_health.return_on_equity, aapl2.financial_health.return_on_equity);
        assert_ne!(aapl1.growth_metrics.revenue_growth_3y, aapl2.growth_metrics.revenue_growth_3y);
        assert_ne!(aapl1.market_sentiment.target_price_consensus, aapl2.market_sentiment.target_price_consensus);
    }

    #[test]
    fn test_different_providers_return_different_msft_data() {
        let mock = crate::providers::mock::MockDataProvider::new();
        let second = SecondMockDataProvider::new();
        let msft1 = mock.get_stock_data("MSFT").unwrap();
        let msft2 = second.get_stock_data("MSFT").unwrap();
        assert_ne!(msft1.valuation_ratios.pe_ratio, msft2.valuation_ratios.pe_ratio);
        assert_ne!(msft1.growth_metrics.revenue_growth_3y, msft2.growth_metrics.revenue_growth_3y);
    }
}

// ============================================================
// Helper Functions Tests
// ============================================================

mod helper_function_tests {
    use crate::models::Recommendation;

    #[test]
    fn test_format_recommendation_strong_buy() {
        assert_eq!(crate::routes::format_recommendation(&Recommendation::StrongBuy), "STRONG BUY");
    }

    #[test]
    fn test_format_recommendation_buy() {
        assert_eq!(crate::routes::format_recommendation(&Recommendation::Buy), "BUY");
    }

    #[test]
    fn test_format_recommendation_hold() {
        assert_eq!(crate::routes::format_recommendation(&Recommendation::Hold), "HOLD");
    }

    #[test]
    fn test_format_recommendation_sell() {
        assert_eq!(crate::routes::format_recommendation(&Recommendation::Sell), "SELL");
    }

    #[test]
    fn test_format_recommendation_strong_sell() {
        assert_eq!(crate::routes::format_recommendation(&Recommendation::StrongSell), "STRONG SELL");
    }

    #[test]
    fn test_format_growth_some_positive() {
        assert_eq!(crate::routes::format_growth(Some(0.112)), "+11.2%");
    }

    #[test]
    fn test_format_growth_some_negative() {
        assert_eq!(crate::routes::format_growth(Some(-0.05)), "-5.0%");
    }

    #[test]
    fn test_format_growth_none() {
        assert_eq!(crate::routes::format_growth(None), "N/A");
    }

    #[test]
    fn test_format_growth_zero() {
        assert_eq!(crate::routes::format_growth(Some(0.0)), "+0.0%");
    }

    #[test]
    fn test_format_ratio_some() {
        assert_eq!(crate::routes::format_ratio(Some(28.5)), "28.50");
    }

    #[test]
    fn test_format_ratio_some_decimal() {
        assert_eq!(crate::routes::format_ratio(Some(3.14159)), "3.14");
    }

    #[test]
    fn test_format_ratio_none() {
        assert_eq!(crate::routes::format_ratio(None), "N/A");
    }

    #[test]
    fn test_format_market_cap_trillions() {
        assert_eq!(crate::routes::format_market_cap(Some(1_500_000_000_000)), "$1.50T");
    }

    #[test]
    fn test_format_market_cap_billions() {
        assert_eq!(crate::routes::format_market_cap(Some(97_500_000_000)), "$97.50B");
    }

    #[test]
    fn test_format_market_cap_millions() {
        assert_eq!(crate::routes::format_market_cap(Some(7_500_000_000)), "$7.50B");
    }

    #[test]
    fn test_format_market_cap_thousands() {
        assert_eq!(crate::routes::format_market_cap(Some(3_500_000)), "$3.50M");
    }

    #[test]
    fn test_format_market_cap_below_million() {
        assert_eq!(crate::routes::format_market_cap(Some(1_800_000)), "$1.80M");
    }

    #[test]
    fn test_format_market_cap_none() {
        assert_eq!(crate::routes::format_market_cap(None), "N/A");
    }

    // health_score_label tests
    #[test]
    fn test_health_score_label_excellent() {
        let (label, color) = crate::routes::health_score_label(0.5, 0.3);
        assert_eq!(label, "Excellent");
        assert_eq!(color, "#10b981");
    }

    #[test]
    fn test_health_score_label_moderate() {
        let (label, color) = crate::routes::health_score_label(0.2, 0.7);
        assert_eq!(label, "Moderate");
        assert_eq!(color, "#f59e0b");
    }

    #[test]
    fn test_health_score_label_weak() {
        let (label, color) = crate::routes::health_score_label(0.05, 2.0);
        assert_eq!(label, "Weak");
        assert_eq!(color, "#ef4444");
    }

    // valuation_assessment tests
    #[test]
    fn test_valuation_assessment_attractive() {
        let (label, color) = crate::routes::valuation_assessment(Some(12.0), Some(8.0));
        assert_eq!(label, "Attractive");
        assert_eq!(color, "#10b981");
    }

    #[test]
    fn test_valuation_assessment_fair() {
        let (label, color) = crate::routes::valuation_assessment(Some(20.0), Some(15.0));
        assert_eq!(label, "Fair");
        assert_eq!(color, "#3b82f6");
    }

    #[test]
    fn test_valuation_assessment_expensive() {
        let (label, color) = crate::routes::valuation_assessment(Some(35.0), Some(25.0));
        assert_eq!(label, "Expensive");
        assert_eq!(color, "#f59e0b");
    }

    #[test]
    fn test_valuation_assessment_overvalued() {
        let (label, color) = crate::routes::valuation_assessment(Some(85.0), Some(45.0));
        assert_eq!(label, "Overvalued");
        assert_eq!(color, "#ef4444");
    }

    #[test]
    fn test_valuation_assessment_defaults_when_none() {
        let (label, color) = crate::routes::valuation_assessment(None, None);
        assert_eq!(label, "Expensive");
        assert_eq!(color, "#f59e0b");
    }

    // growth_assessment tests
    #[test]
    fn test_growth_assessment_strong() {
        let (label, color) = crate::routes::growth_assessment(Some(0.30), Some(0.40));
        assert_eq!(label, "Strong");
        assert_eq!(color, "#10b981");
    }

    #[test]
    fn test_growth_assessment_moderate() {
        let (label, color) = crate::routes::growth_assessment(Some(0.15), Some(0.15));
        assert_eq!(label, "Moderate");
        assert_eq!(color, "#3b82f6");
    }

    #[test]
    fn test_growth_assessment_slow() {
        let (label, color) = crate::routes::growth_assessment(Some(0.08), Some(0.08));
        assert_eq!(label, "Slow");
        assert_eq!(color, "#f59e0b");
    }

    #[test]
    fn test_growth_assessment_declining() {
        let (label, color) = crate::routes::growth_assessment(Some(-0.05), Some(0.02));
        assert_eq!(label, "Declining");
        assert_eq!(color, "#ef4444");
    }

    // create_bar_chart tests
    #[test]
    fn test_create_bar_chart_basic() {
        let data = vec![
            ("P/E".to_string(), 25.0, "#3b82f6".to_string()),
            ("EV/EBITDA".to_string(), 15.0, "#8b5cf6".to_string()),
        ];
        let svg = crate::routes::create_bar_chart(&data, &["#3b82f6", "#8b5cf6"], "Test Chart");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Test Chart"));
    }

    #[test]
    fn test_create_bar_chart_empty_data() {
        let svg = crate::routes::create_bar_chart(&[], &[], "Empty");
        assert!(svg.is_empty());
    }

    #[test]
    fn test_create_bar_chart_contains_labels() {
        let data = vec![
            ("Revenue".to_string(), 10.0, "#3b82f6".to_string()),
            ("EPS".to_string(), 5.0, "#8b5cf6".to_string()),
        ];
        let svg = crate::routes::create_bar_chart(&data, &["#3b82f6", "#8b5cf6"], "Growth");
        assert!(svg.contains("Revenue"));
        assert!(svg.contains("EPS"));
    }

    #[test]
    fn test_create_comparison_chart_basic() {
        let data_a = vec![10.0, 20.0, 30.0];
        let data_b = vec![15.0, 25.0, 35.0];
        let labels = &["A", "B", "C"];
        let titles = &["Provider A", "Provider B"];
        let svg = crate::routes::create_comparison_chart(&data_a, &data_b, labels, titles);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Provider A"));
        assert!(svg.contains("Provider B"));
    }

    #[test]
    fn test_create_comparison_chart_empty_data() {
        let data_a: Vec<f64> = vec![];
        let data_b: Vec<f64> = vec![];
        let svg = crate::routes::create_comparison_chart(&data_a, &data_b, &[], &["A", "B"]);
        assert!(svg.is_empty());
    }

    #[test]
    fn test_create_comparison_chart_contains_labels() {
        let data_a = vec![10.0];
        let data_b = vec![20.0];
        let labels = &["P/E"];
        let titles = &["A", "B"];
        let svg = crate::routes::create_comparison_chart(&data_a, &data_b, labels, titles);
        assert!(svg.contains("P/E"));
    }

    #[test]
    fn test_create_comparison_chart_with_colors() {
        let data_a = vec![10.0];
        let data_b = vec![20.0];
        let svg = crate::routes::create_comparison_chart(&data_a, &data_b, &["A"], &["A", "B"]);
        assert!(svg.contains("#3b82f6"));
        assert!(svg.contains("#f59e0b"));
    }
}

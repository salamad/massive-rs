//! Options-specific models.
//!
//! This module contains types for options market data.

use serde::{Deserialize, Serialize};

/// Option contract details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    /// Option ticker (e.g., "O:AAPL230120C00150000")
    pub ticker: String,
    /// Underlying ticker
    pub underlying_ticker: String,
    /// Contract type (call or put)
    pub contract_type: ContractType,
    /// Expiration date (YYYY-MM-DD)
    pub expiration_date: String,
    /// Strike price
    pub strike_price: f64,
    /// Shares per contract (usually 100)
    pub shares_per_contract: u32,
    /// Exercise style (American or European)
    pub exercise_style: Option<ExerciseStyle>,
    /// Primary exchange
    pub primary_exchange: Option<String>,
    /// Additional exchange listings
    #[serde(default)]
    pub additional_underlyings: Vec<String>,
}

/// Option contract type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractType {
    /// Call option
    Call,
    /// Put option
    Put,
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::Call => write!(f, "call"),
            ContractType::Put => write!(f, "put"),
        }
    }
}

/// Exercise style for options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExerciseStyle {
    /// American style (can exercise any time before expiration)
    American,
    /// European style (can only exercise at expiration)
    European,
}

/// Greeks (option pricing sensitivities).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Greeks {
    /// Delta: Rate of change of option price with respect to underlying
    pub delta: f64,
    /// Gamma: Rate of change of delta with respect to underlying
    pub gamma: f64,
    /// Theta: Rate of change of option price with respect to time
    pub theta: f64,
    /// Vega: Rate of change of option price with respect to volatility
    pub vega: f64,
    /// Rho: Rate of change of option price with respect to interest rate
    pub rho: Option<f64>,
}

/// Option quote with greeks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionQuote {
    /// Option ticker
    pub ticker: String,
    /// Bid price
    pub bid: f64,
    /// Ask price
    pub ask: f64,
    /// Bid size
    pub bid_size: u64,
    /// Ask size
    pub ask_size: u64,
    /// Last price
    pub last: f64,
    /// Mid price
    pub mid: f64,
    /// Open interest
    pub open_interest: Option<u64>,
    /// Implied volatility
    pub implied_volatility: Option<f64>,
    /// Greeks
    pub greeks: Option<Greeks>,
    /// Quote timestamp
    pub timestamp: Option<i64>,
}

impl OptionQuote {
    /// Calculate the bid-ask spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate the spread as percentage of mid.
    pub fn spread_percent(&self) -> f64 {
        if self.mid > 0.0 {
            self.spread() / self.mid * 100.0
        } else {
            0.0
        }
    }
}

/// Option snapshot containing current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionSnapshot {
    /// Break even price
    pub break_even_price: Option<f64>,
    /// Day information
    pub day: Option<OptionDaySnapshot>,
    /// Underlying asset details
    pub details: Option<OptionContract>,
    /// Greeks
    pub greeks: Option<Greeks>,
    /// Implied volatility
    pub implied_volatility: Option<f64>,
    /// Last quote
    pub last_quote: Option<OptionLastQuote>,
    /// Last trade
    pub last_trade: Option<OptionLastTrade>,
    /// Open interest
    pub open_interest: Option<u64>,
    /// Underlying asset
    pub underlying_asset: Option<UnderlyingAsset>,
}

/// Option day aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionDaySnapshot {
    /// Change
    pub change: Option<f64>,
    /// Change percent
    pub change_percent: Option<f64>,
    /// Close price
    pub close: Option<f64>,
    /// High price
    pub high: Option<f64>,
    /// Last updated timestamp
    pub last_updated: Option<i64>,
    /// Low price
    pub low: Option<f64>,
    /// Open price
    pub open: Option<f64>,
    /// Previous close
    pub previous_close: Option<f64>,
    /// Volume
    pub volume: Option<u64>,
    /// VWAP
    pub vwap: Option<f64>,
}

/// Option last quote snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionLastQuote {
    /// Ask price
    pub ask: f64,
    /// Ask size
    pub ask_size: u64,
    /// Bid price
    pub bid: f64,
    /// Bid size
    pub bid_size: u64,
    /// Last updated timestamp
    pub last_updated: Option<i64>,
    /// Midpoint
    pub midpoint: f64,
    /// Timeframe
    pub timeframe: Option<String>,
}

/// Option last trade snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionLastTrade {
    /// Trade conditions
    #[serde(default)]
    pub conditions: Vec<i32>,
    /// Exchange
    pub exchange: Option<u8>,
    /// Price
    pub price: f64,
    /// SIP timestamp
    pub sip_timestamp: Option<i64>,
    /// Size
    pub size: u64,
    /// Timeframe
    pub timeframe: Option<String>,
}

/// Underlying asset information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderlyingAsset {
    /// Change to close
    pub change_to_break_even: Option<f64>,
    /// Last updated timestamp
    pub last_updated: Option<i64>,
    /// Current price
    pub price: Option<f64>,
    /// Ticker
    pub ticker: String,
    /// Timeframe
    pub timeframe: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_type_display() {
        assert_eq!(ContractType::Call.to_string(), "call");
        assert_eq!(ContractType::Put.to_string(), "put");
    }

    #[test]
    fn test_option_quote_spread() {
        let quote = OptionQuote {
            ticker: "O:AAPL230120C00150000".into(),
            bid: 5.00,
            ask: 5.20,
            bid_size: 100,
            ask_size: 50,
            last: 5.10,
            mid: 5.10,
            open_interest: Some(1000),
            implied_volatility: Some(0.30),
            greeks: Some(Greeks {
                delta: 0.5,
                gamma: 0.02,
                theta: -0.01,
                vega: 0.1,
                rho: Some(0.05),
            }),
            timestamp: Some(1703001234567),
        };

        assert!((quote.spread() - 0.20).abs() < 0.001);
        assert!((quote.spread_percent() - 3.92).abs() < 0.1);
    }

    #[test]
    fn test_option_contract_deserialize() {
        let json = r#"{
            "ticker": "O:AAPL230120C00150000",
            "underlying_ticker": "AAPL",
            "contract_type": "call",
            "expiration_date": "2023-01-20",
            "strike_price": 150.0,
            "shares_per_contract": 100,
            "exercise_style": "american"
        }"#;

        let contract: OptionContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.ticker, "O:AAPL230120C00150000");
        assert_eq!(contract.contract_type, ContractType::Call);
        assert_eq!(contract.strike_price, 150.0);
    }

    #[test]
    fn test_greeks_deserialize() {
        let json = r#"{
            "delta": 0.5,
            "gamma": 0.02,
            "theta": -0.01,
            "vega": 0.1,
            "rho": 0.05
        }"#;

        let greeks: Greeks = serde_json::from_str(json).unwrap();
        assert_eq!(greeks.delta, 0.5);
        assert_eq!(greeks.gamma, 0.02);
    }
}

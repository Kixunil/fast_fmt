//! This module contains consts for strategies. It may be handy to get a reference to strategy with
//! `'static` llifetime.

use ::{Display, Debug};

/// Display strategy.
pub static DISPLAY: Display = Display;

/// Debug strategy.
pub static DEBUG: Debug = Debug;

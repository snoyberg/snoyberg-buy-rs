extern crate time;

#[derive(Debug)]
enum BuyError {
    MissingEnvVar(&'static str, std::env::VarError),
    InsufficientArgs(usize),
    TooManyArgs(usize),
    InvalidExpense(String),
    InvalidAmount(String),
    IO(std::io::Error),
}

impl From<std::io::Error> for BuyError {
    fn from(e: std::io::Error) -> BuyError {
        BuyError::IO(e)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Expense {
    Shufersal,
    KeterHabasar,
    TalTavlinim,
}

impl Expense {
    fn parse(s: String) -> Result<Expense, BuyError> {
        match s.as_ref() {
            "shufersal" => Ok(Expense::Shufersal),
            "keter" => Ok(Expense::KeterHabasar),
            "tal" => Ok(Expense::TalTavlinim),
            _ => Err(BuyError::InvalidExpense(s)),
        }
    }

    fn desc(&self) -> &'static str {
        match self {
            Expense::Shufersal => "Shufersal",
            Expense::KeterHabasar => "Keter Habasar",
            Expense::TalTavlinim => "Tal Tavlinim",
        }
    }

    fn dest_account(&self) -> &'static str {
        match self {
            Expense::Shufersal => "expenses:food",
            Expense::KeterHabasar => "expenses:food",
            Expense::TalTavlinim => "expenses:food",
        }
    }

    fn source_account(&self) -> &'static str {
        match self {
            Expense::Shufersal => "liability:credit card:fibi:shufersal",
            Expense::KeterHabasar => "liability:credit card:fibi:shufersal",
            Expense::TalTavlinim => "liability:credit card:fibi:shufersal",
        }
    }

    fn fmt<F>(&self, fmt: &mut F, amount: u32, now: time::Tm) -> Result<(), std::io::Error>
    where
        F: std::io::Write,
    {
        write!(
            fmt,
            "\n{}/{:02}/{:02} {}\n    {}  ₪{}\n    {}\n",
            now.tm_year + 1900,
            now.tm_mon + 1,
            now.tm_mday,
            self.desc(),
            self.dest_account(),
            amount,
            self.source_account(),
        )
    }
}

fn main() -> Result<(), BuyError> {
    let ledger_file = get_ledger_file()?;
    let (_exename, expense_str, amount_str) = require_three(std::env::args())?;
    let expense = Expense::parse(expense_str)?;
    let amount = match u32::from_str_radix(&amount_str, 10) {
        Ok(amount) => amount,
        Err(_) => return Err(BuyError::InvalidAmount(amount_str)),
    };

    let mut options = std::fs::OpenOptions::new();
    let mut file = options.append(true).open(ledger_file)?;

    expense.fmt(&mut file, amount, time::now())?;

    Ok(())
}

const LEDGER_VAR: &'static str = "LEDGER_FILE";

fn get_ledger_file() -> Result<String, BuyError> {
    std::env::var(LEDGER_VAR).map_err(|e| BuyError::MissingEnvVar(LEDGER_VAR, e))
}

fn require_three<I>(mut iter: I) -> Result<(String, String, String), BuyError>
where
    I: Iterator<Item = String>,
{
    let x = match iter.next() {
        Some(x) => x,
        None => return Err(BuyError::InsufficientArgs(0)),
    };
    let y = match iter.next() {
        Some(y) => y,
        None => return Err(BuyError::InsufficientArgs(1)),
    };
    let z = match iter.next() {
        Some(z) => z,
        None => return Err(BuyError::InsufficientArgs(2)),
    };
    let rest = iter.count();
    if rest == 0 {
        Ok((x, y, z))
    } else {
        Err(BuyError::TooManyArgs(rest + 3))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_ledger_file() {
        std::env::remove_var(LEDGER_VAR);
        match get_ledger_file() {
            Err(BuyError::MissingEnvVar(var, _)) => assert_eq!(var, LEDGER_VAR),
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_has_ledger_file() {
        let val = "foobarbazbin";
        std::env::set_var(LEDGER_VAR, val);
        assert_eq!(get_ledger_file().unwrap(), val);
    }

    #[test]
    fn test_require_three() {
        assert_eq!(
            require_three(vec!["1", "2", "3"].into_iter().map(|s| String::from(s))).unwrap(),
            ("1".to_string(), "2".to_string(), "3".to_string())
        );
        match require_three((vec![] as Vec<String>).into_iter()) {
            Err(BuyError::InsufficientArgs(0)) => (),
            x => panic!("{:?}", x),
        }
        match require_three(vec!["1"].into_iter().map(|s| String::from(s))) {
            Err(BuyError::InsufficientArgs(1)) => (),
            x => panic!("{:?}", x),
        }
        match require_three(
            vec!["1", "2", "3", "4"]
                .into_iter()
                .map(|s| String::from(s)),
        ) {
            Err(BuyError::TooManyArgs(4)) => (),
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_expense_parse() {
        assert_eq!(
            Expense::parse("shufersal".to_string()).unwrap(),
            Expense::Shufersal
        );
    }

    #[test]
    fn test_fmt() {
        let mut vec = vec![];
        let tm = time::at(time::Timespec::new(0, 0));
        Expense::KeterHabasar.fmt(&mut vec, 100, tm).unwrap();
        let s = String::from_utf8(vec).unwrap();
        assert_eq!(s, "\n1970/01/01 Keter Habasar\n    expenses:food  ₪100\n    liability:credit card:fibi:shufersal\n");
    }
}

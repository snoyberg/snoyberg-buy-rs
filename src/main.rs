extern crate time;
extern crate gtk;
extern crate gio;
extern crate glib;

use gtk::prelude::*;
use gio::prelude::*;
use std::fs::File;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
enum BuyError {
    MissingEnvVar(&'static str, std::env::VarError),
    IO(std::io::Error),
    GtkLaunch(glib::error::BoolError),
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

    fn fmt<F>(&self, fmt: &mut F, amount: &str, now: time::Tm) -> Result<(), std::io::Error>
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
    let mut options = std::fs::OpenOptions::new();
    let file = Rc::new(RefCell::new(options.append(true).open(ledger_file)?));

    let application = match gtk::Application::new("com.github.snoyberg.snoyberg-buy-rs",gio::ApplicationFlags::empty()) {
        Ok(app) => app,
        Err(e) => return Err(BuyError::GtkLaunch(e)),
    };
    application.connect_startup(move |app| {
        // Story time, since I was stumped. Why do we need to pass in
        // a reference to the file here? Why not pass the file itself?
        // Alternatively, we could do file.clone() here, but why is
        // _that_ necessary?
        //
        // Without one of those changes, I get the error message:
        //
        // > cannot move out of captured outer variable in an `Fn` closure
        //
        // Eventually I found this link:
        //
        // https://stackoverflow.com/questions/33662098/cannot-move-out-of-captured-outer-variable-in-an-fn-closure
        //
        // Which provides the missing piece: an `Fn` closure can be
        // called multiple times. Therefore, with a non-reference and
        // no clone, the first call to this closure will pass
        // ownership of the single copy of `file` to `build_ui`, and
        // later calls won't have it available. When we pass by
        // reference, the closure itself retains ownership. When we
        // clone, then each call gets its own copy.
        build_ui(app, &file)
    });
    application.connect_activate(|_| {});
    application.run(&std::env::args().collect::<Vec<_>>());

    Ok(())
}

fn build_ui(application: &gtk::Application, file_cell: &Rc<RefCell<File>>) {
    let expenses = vec![Expense::Shufersal, Expense::KeterHabasar, Expense::TalTavlinim];
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Buy stuff, mostly food");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70 + (expenses.len() as i32) * 50);

    // FIXME Cmd-Q handling
    // FIXME should pop up over other apps (focus stealing?)

    window.connect_delete_event(move |win, _| {
        win.destroy();
        Inhibit(false)
    });

    let container = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let label = gtk::Label::new("Amount");
    container.add(&label);

    let spin = gtk::SpinButton::new_with_range(0.0, 10000000.0, 0.01);
    container.add(&spin);

    for e in expenses.into_iter() {
        let button = gtk::Button::new_with_label(e.desc());
        let spin = spin.clone();
        let window = window.clone();
        let file_cell = file_cell.clone();
        button.connect_clicked(move |_button| {
            let msg = (|| {
                let mut file = match file_cell.try_borrow_mut() {
                    Ok(mut file) => file,
                    Err(e) => return format!("Could not borrow the file: {}", e),
                };
                let amount = match spin.get_text() {
                    None => return String::from("No amount available"),
                    Some(amount) => amount
                };
                match e.fmt(&mut *file, &amount, time::now()) {
                    Ok(()) => format!("Spent ₪{} on {}", &amount, e.desc()),
                    Err(e) => format!("Could not write to the file: {}", e),
                }
            })();
            let dialog = gtk::MessageDialog::new(
                Some(&window),
                gtk::DialogFlags::empty(),
                gtk::MessageType::Info,
                gtk::ButtonsType::Ok,
                &msg);
            dialog.run();
            dialog.destroy();
        });
        container.add(&button);
    }

    window.add(&container);
    window.show_all();
}

const LEDGER_VAR: &'static str = "LEDGER_FILE";

fn get_ledger_file() -> Result<String, BuyError> {
    std::env::var(LEDGER_VAR).map_err(|e| BuyError::MissingEnvVar(LEDGER_VAR, e))
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
    fn test_fmt() {
        let mut vec = vec![];
        let tm = time::at(time::Timespec::new(0, 0));
        Expense::KeterHabasar.fmt(&mut vec, "100", tm).unwrap();
        let s = String::from_utf8(vec).unwrap();
        assert_eq!(s, "\n1970/01/01 Keter Habasar\n    expenses:food  ₪100\n    liability:credit card:fibi:shufersal\n");
    }
}

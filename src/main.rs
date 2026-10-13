mod app;
mod clipboard;
mod fonts;
mod locale;
mod output;
mod parse;

use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

use eframe::egui;

const USAGE: &str = "\
usage: epochup              open the popup on the clipboard's contents
       epochup TEXT...      open the popup with TEXT typed in
       epochup -e TEXT...   print the conversion of TEXT (stdin with -) and exit

In the popup: Enter copies the highlighted value and closes, arrows and Tab
move the highlight, Alt+1..9 copy an alternative. Esc first sets the clipboard
aside to show the current time, then closes.";

fn main() -> eframe::Result {
    locale::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some(clipboard::SERVE_FLAG) => {
            let mut text = String::new();
            if std::io::stdin().read_to_string(&mut text).is_ok() {
                clipboard::serve(text);
            }
            return Ok(());
        }
        Some("-e" | "--eval") => {
            eval(&args[1..]);
            return Ok(());
        }
        Some("-h" | "--help") => {
            println!("{USAGE}");
            return Ok(());
        }
        _ => {}
    }

    // Read the font and the clipboard while the window is being created.
    let font = std::thread::spawn(fonts::load_cjk);
    let input = args.join(" ");
    let clip_rx = input.is_empty().then(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || tx.send(clipboard::startup_text()));
        rx
    });

    let copied = Rc::new(RefCell::new(None));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("epochup")
            .with_title("epochup")
            .with_inner_size([900.0, 230.0])
            .with_decorations(false),
        ..Default::default()
    };
    let app_copied = copied.clone();
    eframe::run_native(
        "epochup",
        options,
        Box::new(move |cc| {
            fonts::install(&cc.egui_ctx, font.join().ok().flatten());
            Ok(Box::new(app::App::new(cc, input, clip_rx, app_copied)))
        }),
    )?;

    if let Some(text) = copied.take()
        && let Err(e) = clipboard::hand_off(&text)
    {
        eprintln!("epochup: copying to the clipboard: {e}");
    }
    Ok(())
}

fn eval(args: &[String]) {
    let mut text = args.join(" ");
    if text.is_empty() || text == "-" {
        text.clear();
        if let Err(e) = std::io::stdin().read_to_string(&mut text) {
            eprintln!("epochup: reading stdin: {e}");
            std::process::exit(2);
        }
    }
    let ctx = parse::Ctx::system();
    let Some(it) = parse::interpret(&text, &ctx) else {
        eprintln!("epochup: no date or epoch found");
        std::process::exit(1);
    };
    let out = output::build(&it, &text, &ctx);
    println!("{}", out.primary.text);
    for alt in &out.alternatives {
        println!("  {:<34} {}", alt.label, alt.text);
    }
    println!("# {}", out.info);
}

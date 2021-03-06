use clap::{App, Arg, ArgMatches};

pub fn parse<'a>() -> ArgMatches<'a> {
    App::new("fxconv")
        .version("0.1.0")
        .author("Nicholas Buckeridge <bucknich@gmail.com>")
        .about([
            "fxtickconv converts tick data to the desired output timeframe formatted to ",
            "specification. The input and output data columns must/are comma seperated.",
            "Conditions:",
            " - If there is more than one file then they must be named so that they can be",
            "      ordered.",
            ""].join("\n").as_str())
        .arg(Arg::with_name("timeframe")
            .index(1)
            .required(true)
            .value_name("TIMEFRAME")
            .help("Specify the output timeframe (eg. \"1m\" is one minute.)")
            .long_help("The time frame is specified by a number followed directly by a single \
            character. The number describes how long the unit in time is and the character is the \
            type of unit. The units are as follows:\n\
            \ts\tSeconds\n\
            \tm\tMinutes\n\
            \th\tHours\n\
            \td\tDays\n\
            \tw\tWeeks\n\
            \tn\tMonths\n\
            \ty\tYears")
        )
        .arg(Arg::with_name("output")
            .index(2)
            .required(true)
            .value_name("OUTPUT")
            .help("The file name to export the data to")
        )
        .arg(Arg::with_name("inputs")
            .index(3)
            .multiple(true)
            .required(true)
            .value_name("INPUTS")
            .help("The input data file/s. All input files must be of the same format")
        )
        .arg(Arg::with_name("overwrite")
            .long("overwrite")
            .short("o")
            .conflicts_with("no-overwrite")
            .help("Force output overwrite if output file already exists.")
        )
        .arg(Arg::with_name("no-overwrite")
            .long("no-overwrite")
            .short("n")
            .conflicts_with("overwrite")
            .help("Exit with error when the output file already exists.")
        )
        .arg(Arg::with_name("ask-only")
            .long("ask-only")
            .short("a")
            .conflicts_with_all(&["bid-only", "bid-first"])
            .help("Export ask data only.")
        )
        .arg(Arg::with_name("bid-only")
            .long("bid-only")
            .short("b")
            .conflicts_with_all(&["ask-only", "ask-first"])
            .help("Export bid data only.")
        )
        .arg(Arg::with_name("ask-first")
            .long("ask-first")
            .short("c")
            .conflicts_with_all(&["bid-only", "bid-first"])
            .help("Place ask columns before the bid columns.")
        )
        .arg(Arg::with_name("bid-first")
            .long("bid-first")
            .short("d")
            .conflicts_with_all(&["ask-only", "ask-first"])
            .help("Place bid columns before the ask columns.")
        )
        .arg(Arg::with_name("headers")
            .long("headers")
            .short("h")
            .help("Prepend commented (#) lines describing the data at the top of the file")
        )
        .arg(Arg::with_name("tick")
            .long("tick")
            .short("t")
            .takes_value(true)
            .default_value("xdab")
            .required(true)
            .help("Informs the converter that the input data is tick data")
            .long_help([
                "Informs the converter that the input data is tick data. When this option ",
                "is used, the format of the data must be specified. Specifically the date, ",
                "ask and bid columns. Format options:\n",
                "    Option  Description\n",
                "    d       datetime (must have one)\n",
                "    a       ask (must have one)\n",
                "    b       bid (must have one)\n",
                "    x       column filler\n"
            ].join("").as_str())
        )
        .arg(Arg::with_name("bar")
            .long("bar")
            .help("Show progress bar")
        )
    .get_matches()
}

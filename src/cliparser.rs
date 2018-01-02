use clap::{App, Arg};

pub fn parse() {
    let matches = App::new("fxconv")
        .version("0.1.0")
        .author("Nicholas Buckeridge <bucknich@gmail.com>")
        .about([
            "fxconv converts ohlc (or tick) data from an input timeframe to the desired",
            "output timeframe formatted to specification. The output is streamed to stdout",
            "unless specified otherwise. When there are time gaps in the data, the timeframe",
            "at the start of the gap will be trimmed and begin at the end of the gap.",
            "",
            "Conditions:",
            " - The input files must have the same timeframe and be specified in order (when the ",
            "file ends at a time, the next file must start at the next time)",
            " - Output timeframe must be smaller than the input timeframe.",
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
        .arg(Arg::with_name("input-delimiter")
            .long("input-delimiter")
            .short("i")
            .takes_value(true)
            .value_name("DELIMITER")
            .help("Input delimiter (whitespace is default)")
        )
        .arg(Arg::with_name("output-delimiter")
            .long("output-delimiter")
            .short("o")
            .takes_value(true)
            .value_name("DELIMITER")
            .help("Output delimiter (tab is default)")
        )
        .arg(Arg::with_name("overwrite")
            .long("overwrite")
            .short("w")
            .help("Force output overwrite if output file already exists.")
        )
        .arg(Arg::with_name("no-overwrite")
            .long("no-overwrite")
            .short("n")
            .help("Exit with error when the output file already exists.")
        )
        .arg(Arg::with_name("ask-only")
            .long("ask-only")
            .short("a")
            .help("Export ask data only.")
        )
        .arg(Arg::with_name("bid-only")
            .long("bid-only")
            .short("b")
            .help("Export bid data only.")
        )
        .arg(Arg::with_name("ask-first")
            .long("ask-first")
            .short("c")
            .help("Place ask columns before the bid columns.")
        )
        .arg(Arg::with_name("bid-first")
            .long("bid-first")
            .short("d")
            .help("Place bid columns before the ask columns.")
        )
        .arg(Arg::with_name("format")
            .long("format")
            .short("f")
            .takes_value(true)
            .value_name("FORMAT")
            .help("Format specifier string for the output, of which describes the format of each \
                line")
            .long_help([
                "Format specifier string for the output, of which describes the format ",
                "of each line. The line will always start with the date (index), then ",
                "it will follow the format specifier for the ask, then bid data (unless ",
                "a flag changes that behaviour.) The formatting options are as follows:\n",
                "    Option  Description\n",
                "    o       open\n",
                "    h       high\n",
                "    l       low\n",
                "    c       close\n",
                "    a       mean\n",
                "    m       meadian\n",
                "    e       mode\n",
                "By default, the format specifier is \"ohlc\""
            ].join("").as_str())
        )
        .arg(Arg::with_name("tick")
            .long("tick")
            .short("t")
            .help("Informs the converter that the input data is tick data")
            .long_help([
                "Informs the converter that the input data is tick data. When this option ",
                "is used, the format of the data must be specified. Specifically the date, ",
                "ask and bid columns. Format options:\n",
                "    Option  Description\n",
                "    d       datetime\n",
                "    a       ask\n",
                "    b       bid\n",
                "    x       column filler"
            ].join("").as_str())
        )
        .arg(Arg::with_name("headers")
            .long("headers")
            .short("h")
            .help("Prepend commented (#) lines describing the data at the top of the file")
        )
        .arg(Arg::with_name("precision")
            .long("precision")
            .short("p")
            .takes_value(true)
            .value_name("PRECISION")
            .help("Number of decimal places to allow per data column")
        )
        .arg(Arg::with_name("start")
            .long("start")
            .short("s")
            .takes_value(true)
            .value_name("DATETIME")
            .help("Specify the time to begin the timeframe series")
            .long_help([
                "Specify the time to begin the timeframe series. The specifiers can be as ",
                "follows:\n",
                "\"day/month/year\"\n",
                "    e.g. \"11/06/1996\"\n",
                "\"hour:minute:second\"\n",
                "    e.g. \"15:55:00\"\n",
                "\"day/month/year hour:minute:second\"\n",
                "    e.g. \"11/06/1996 15:55:00\"\n"
            ].join("").as_str())
        )
        .arg(Arg::with_name("end")
            .long("end")
            .short("e")
            .takes_value(true)
            .value_name("DATETIME")
            .help("Specify the time to end the timeframe series")
            .long_help([
                "Specify the time to end the timeframe series. The specifiers can be as ",
                "follows:\n",
                "\"day/month/year\"\n",
                "    e.g. \"11/06/1996\"\n",
                "\"hour:minute:second\"\n",
                "    e.g. \"15:55:00\"\n",
                "\"day/month/year hour:minute:second\"\n",
                "    e.g. \"11/06/1996 15:55:00\"\n"
            ].join("").as_str())
        )
        .arg(Arg::with_name("gaps")
            .long("gaps")
            .short("g")
            .default_value("skip")
            .possible_values(&["skip", "continue", "skip-weekends", "stop"])
            .help("Specify the action to take when encountering a gap in timeframes")
            .long_help([
                "Specify the action to take when encountering a gap in timeframes. ",
                "The actions inclue:\n",
                "    skip - skip the missing timeframes\n",
                "    continue - fill in the missing timeframes with the last price\n",
                "    skip-weekends - only skip the weekends, continue price during the week\n",
                "    stop - stop when a gap in timeframes is detected, stop the program with an ",
                "error\n",
            ].join("").as_str())
        )
        .get_matches();
}



/*
Lets just understand how the program is going to work.
There is a converter object that converts the input stream to the output stream.
istream >> converter >> ostream.
the converter needs to be told input format and the desired output format
*/

/*
fxconv converts ohlc (or tick) data from an input timeframe to the desired output timeframe.

Stream directors
--export_to_file=file_name
--export_to_stdout (default)


Format specifiers
--in_type=<csv|dat>
--out_type=<csv|dat>
--in_format=""
--out_format=""
--tick_in
--ohlc_out
--ohlc_in
--headers
--decimals,d=5

Formating Options
    d   datetime
    o   open
    h   high
    l   low
    c   close
    a   mean
    m   meadian
    e   mode


Data range
-s <time>, --start="<time>"
-e <time>, --end="<time>"

--bs=<size>
--threads=number


-h, --help

<input files in sequence>
*/

#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <cstdlib>
#include <exception>

using std::cout;
using std::cerr;
using std::endl;
using std::string;
using std::vector;
using std::ofstream;
using std::runtime_error;
using std::stringstream;
using std::cin;
using std::getline;


#include "application.hpp"
#include <boost/program_options.hpp>
#include <boost/filesystem/path.hpp>
#include <boost/filesystem.hpp>

using boost::program_options::options_description;
using boost::program_options::variables_map;
using boost::program_options::value;
using boost::program_options::parse_command_line;
using boost::program_options::store;
using boost::program_options::notify;
using boost::program_options::error;
using boost::filesystem::path;
using boost::filesystem::is_directory;
using boost::filesystem::is_regular_file;
using boost::filesystem::exists;

/**
 * Application constructor. Extract the arguments and prepare the program.
 */
Application::Application(int argc, char const* argv[])
{
    // Unpack command line arguments
    try
    {
        options_description desc{
            "Usage: fxconv [options] <input files>\n"
            "\n"
            "fxconv converts ohlc (or tick) data from an input timeframe to the desired\n"
            "output timeframe formatted to specification. The output is streamed to stdout\n"
            "unless specified otherwise. When there are time gaps in the data, the timeframe\n"
            "at the start of the gap will be trimmed and begin at the end of the gap.\n"
            "\n"
            "Conditions\n"
            "==========\n"
            " - The input files must have the same timeframe.\n"
            " - Output timeframe must be smaller than the input timeframe.\n"
            " - If there is more than one file then they must be named so that they can be\n"
            "      ordered.\n"
            "\n"
            "Options"
        };
        desc.add_options()
            ("help,h", "Help screen.")
            ("export", value<string>(), "Export to path/to/file (stdout is default).")
            ("input-delimiter", value<string>(), "Input delimiter (whitespace is default).")
            ("output-delimiter", value<string>(), "Output delimiter (tab is default).")
            ("overwrite", "Force output overwrite if output file already exists.")
            ("no-overwrite", "Exit with error when the output file already exists.")
            ("ask-only", "Export ask data only.")
            ("bid-only", "Export bid data only.")
            ("ask-first", "Place ask columns before the bid columns.")
            ("bid-first", "Place bid columns before the ask columns.")
            ("format,f", value<string>()->default_value("ohlc"),
                "Format specifier string for the output, of which describes the format "
                "of each line. The line will always start with the date (index), then "
                "it will follow the format specifier for the ask, then bid data (unless "
                "a flag changes that behaviour.) The formatting options are as follows:\n"
                "    Option  Description\n"
                "    o       open\n"
                "    h       high\n"
                "    l       low\n"
                "    c       close\n"
                "    a       mean\n"
                "    m       meadian\n"
                "    e       mode\n"
                "By default, the format specifier is \"ohlc\""
            )
            ("tick", value<string>(),
                "Informs the converter that the input data is tick data. When this option "
                "is used, the format of the data must be specified. Specifically the date, "
                "ask and bid columns. Format options:\n"
                "    Option  Description\n"
                "    d       datetime\n"
                "    a       ask\n"
                "    b       bid\n"
                "    x       column filler"
            )
            ("headers", "Prepend commented (#) lines describing the data at the top of the file.")
            ("decimals,d", value<unsigned int>()->default_value(5),
                "Number of decimal places to allow per data column."
            )
            ("start,s", value<string>(),
                "Specify the time to begin the timeframe series. The specifiers can be as "
                "follows:\n"
                "\"day/month/year\"\n"
                "    e.g. \"11/06/1996\"\n"
                "\"hour:minute:second\"\n"
                "    e.g. \"15:55:00\"\n"
                "\"day/month/year hour:minute:second\"\n"
                "    e.g. \"11/06/1996 15:55:00\"\n"
            )
            ("end,e", value<string>(),
                "Similar to the --start argument except describes when the timeframe series "
                "will end."
            )
            ("buffer-size", value<string>()->default_value("1M"),
                "Set the file reading block size. This argument should be a number followed "
                "by either a 'K' for kilobytes, 'M' for megabytes or 'G' for gigabytes."
            )
            ("threads", value<unsigned int>()->default_value(1),
                "Number of threads to use in conversion process."
            )
            ("order", "...")
        ;
        variables_map vm;
        store(parse_command_line(argc, argv, desc), vm);
        notify(vm);

        // print help screen and exit
        if (vm.count("help"))
        {
            cout << desc << endl;
            exit(0);
        }

        // check that overwrite and nooverwrite are note used together
        if (vm.count("overwrite") && vm.count("nooverwrite"))
        {
            throw runtime_error("Cannot use options --overwrite and --nooverwrite together.");
        }

        // Set the output stream correctly and exit
        if (vm.count("export"))
        {
            string path_str = vm["export"].as<string>();
            path p(path_str);
            if (exists(p))
            {
                if (is_regular_file(p))
                {
                    if (vm.count("nooverwrite")) exit(1);
                    if (!vm.count("overwrite"))
                    {
                        cerr << p << " already exists, overwrite? (Yes/no)? ";
                        string response;
                        getline(cin, response);
                        if (!(
                            response == "Yes" ||
                            response == "Y"   ||
                            response == "y"   ||
                            response == "yes"
                        )) exit(0);
                    }
                }
                else if (is_directory(p))
                {
                    stringstream msg;
                    msg << path_str <<": Is a directory";
                    throw runtime_error(msg.str().c_str());
                }
                else
                {
                    stringstream msg;
                    msg << path_str <<": Not a regular file.";
                    throw runtime_error(msg.str().c_str());
                }
            }

            ofs.open(path_str);
            if (!ofs.is_open()) throw runtime_error("Failed opening file.");
            out.rdbuf(ofs.rdbuf());
        }
    }
    catch (const error &ex)
    {
        cerr << "fxconv: " << ex.what() << endl;
        exit(1);
    }
    catch (const runtime_error &ex)
    {
        cerr << "fxconv: " << ex.what() << endl;
        exit(1);
    }

}

/**
 * Application deconstructor.
 */
Application::~Application()
{
    if (ofs.is_open()) {
        ofs.close();
    }
}

/**
 * Run the application and return the program result.
 */
int Application::run()
{

    out << "test" << endl;
    return 0;
}

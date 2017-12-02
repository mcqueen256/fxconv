

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
        options_description desc{"Options"};
        desc.add_options()
            ("help,h", "Help screen.")
            ("export,e", value<string>(), "Export to path/to/file (stdout is default).")
            ("idelimiter", value<string>(), "Input delimiter (whitespace is default).")
            ("overwrite", "Force output overwrite if output file already exists.")
            ("nooverwrite", "Exit with error when the output file already exists.")
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

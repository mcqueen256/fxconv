#include <ctime>
#include <iostream>
#include <string>
#include <vector>
#include <functional>
#include <fstream>

using std::ostream;
using std::cout;
using std::string;
using std::vector;
using std::function;
using std::ofstream;

enum class TimeFrameUnit
{
    Unspecified,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month
};

class Application
{
private:
    int tf_value = 0;
    TimeFrameUnit tf_unit = TimeFrameUnit::Unspecified;
    string* input_file_path = NULL;
    string* output_file_path = NULL;

    ofstream ofs;
    ostream out{cout.rdbuf()};

    function<void(void)> ostream_cleanup{[](){}};

public:
    Application(int argc, char const* argv[]);
    ~Application();

    int run();

private:
};

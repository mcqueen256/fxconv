#include <string>
#include <vector>

#include "application.hpp"

/**
 * Main program entry point.
 */
int main(int argc, char const *argv[]) {
    Application app(argc, argv);
    return app.run();
}

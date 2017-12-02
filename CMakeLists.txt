cmake_minimum_required(VERSION 3.0)

set(CMAKE_BUILD_TYPE Debug)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++14")

project(fxconv)

FIND_PACKAGE( Boost 1.65 COMPONENTS program_options filesystem REQUIRED )
INCLUDE_DIRECTORIES( ${Boost_INCLUDE_DIR} )

add_executable(
    fxconv
    main.cpp
    application.cpp
)

TARGET_LINK_LIBRARIES( fxconv LINK_PUBLIC ${Boost_LIBRARIES} )

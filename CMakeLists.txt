cmake_minimum_required(VERSION 3.0)
project(fxconv)

###############################################################################
# fxconv Version
###############################################################################

set(FXCONV_VERSION_MAJOR 0)
set(FXCONV_VERSION_MINOR 1)
set(FXCONV_VERSION_PATCH 0)
set(FXCONV_VERSION "${FXCONV_VERSION_MAJOR}.${FXCONV_VERSION_MINOR}.${FXCONV_VERSION_PATCH}")

###############################################################################
# fxconv Files
###############################################################################

set(FXCONV_SOURCE_DIR ${PROJECT_SOURCE_DIR}/src)
set(FXCONV_INCLUDE_DIR ${PROJECT_SOURCE_DIR}/include)
set(FXCONV_TEST_DIR ${PROJECT_SOURCE_DIR}/test)

set(FXCONV_SOURCE_FILES
    ${FXCONV_SOURCE_DIR}/main.cpp
    ${FXCONV_SOURCE_DIR}/application.cpp
)

set(FXCONV_HEADER_FILES
    ${FXCONV_INCLUDE_DIR}/application.hpp
)

###############################################################################
# Libraries
###############################################################################

find_package( Boost 1.65 COMPONENTS program_options filesystem REQUIRED )

###############################################################################
# Compiler options
###############################################################################

set(CMAKE_BUILD_TYPE Debug)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++14")

add_executable(fxconv ${FXCONV_SOURCE_FILES} ${FXCONV_HEADER_FILES})
target_include_directories(fxconv PUBLIC $<BUILD_INTERFACE:${FXCONV_INCLUDE_DIR}>)
target_include_directories(fxconv PUBLIC $<BUILD_INTERFACE:${Boost_INCLUDE_DIR}>)
target_compile_definitions(fxconv PUBLIC BOOST_ALL_NO_LIB)

TARGET_LINK_LIBRARIES( fxconv LINK_PUBLIC ${Boost_LIBRARIES} )

target_compile_options(fxconv PRIVATE $<$<CXX_COMPILER_ID:MSVC>:/W4>)
target_compile_options(fxconv PRIVATE
    $<$<OR:$<CXX_COMPILER_ID:AppleClang>,$<CXX_COMPILER_ID:Clang>>:-pedantic -Wall>)
target_compile_options(fxconv PRIVATE $<$<CXX_COMPILER_ID:GNU>:-pedantic -Wall>)

set_property(TARGET fxconv PROPERTY VERSION ${FXCONV_VERSION})
set_property(TARGET fxconv PROPERTY SOVERSION ${FXCONV_VERSION_MAJOR})
set_property(TARGET fxconv PROPERTY DEBUG_POSTFIX d)

cmake_minimum_required(VERSION 3.10)

#设置项目名称
project(libco_base)

#包含头文件目录
include_directories(${CMAKE_SOURCE_DIR}/include/libco ${CMAKE_SOURCE_DIR}/include/cxxopts)

#设置C++标准
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED True)

#添加可执行文件
add_executable(libco_base src/main.cpp)

#添加库文件目录
link_directories(${CMAKE_SOURCE_DIR}/include/libco)

#链接libco库
target_link_libraries(libco_base libcolib)

# CMakeList.txt: OpenCV_ImageTestCard 的 CMake 项目，在此处包括源代码并定义
# 项目特定的逻辑。
#

# 将源代码添加到此项目的可执行文件。
add_executable (OpenCV_ImageTestCard "main.cpp" "OpenCV_ImageTestCard.cpp" "OpenCV_ImageTestCard.h")

if (CMAKE_VERSION VERSION_GREATER 3.12)
  set_property(TARGET OpenCV_ImageTestCard PROPERTY CXX_STANDARD 20)
endif()

# TODO: 如有需要，请添加测试并安装目标。
set(OpenCV_DIR "C:/Program Files/opencv4130/opencv/build/x64/vc16/lib")
find_package(OpenCV REQUIRED CONFIG)
include_directories(${OpenCV_INCLUDE_DIRS})
target_link_libraries(OpenCV_ImageTestCard ${OpenCV_LIBS})

# 将 opencv_world4130.dll 复制到可执行文件同目录，确保运行时可找到 DLL
set(OpenCV_BIN_DIR "C:/Program Files/opencv4130/opencv/build/x64/vc16/bin" CACHE PATH "Path to OpenCV bin directory")
add_custom_command(TARGET OpenCV_ImageTestCard POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${OpenCV_BIN_DIR}/$<IF:$<CONFIG:Debug>,opencv_world4130d.dll,opencv_world4130.dll>"
            $<TARGET_FILE_DIR:OpenCV_ImageTestCard>)
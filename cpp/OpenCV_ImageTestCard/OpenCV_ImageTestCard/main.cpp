#include "OpenCV_ImageTestCard.h"
#include <algorithm>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>

int test_get_img_center(const std::string &imagePath, int index, int sz_min, int sz_max,
                        int num_max)
{
    std::vector<unsigned char> image;
    int width = 0, height = 0;
    if (!load_img_data(imagePath, image, width, height))
    {
        std::cerr << "Failed to load image at " << imagePath << std::endl;
        return -1;
    }

    int center_points[100] = {0}; // 假设最多检测到100个正方形
    num_max = std::clamp(num_max, 0, 100); // 不超过实际缓冲区的 int 元素数
    int center_count = 0;

    get_img_obj_center(image.data(), width, height, index, center_points, num_max, sz_min, sz_max, &center_count);
    std::cout << "Detected " << center_count << " objects." << std::endl;
    get_img_rect_center(image.data(), width, height, index, center_points, num_max, sz_min, sz_max, &center_count);
    std::cout << "Detected " << center_count << " squares." << std::endl;
    get_img_circle_center(image.data(), width, height, index, center_points, num_max, sz_min, sz_max, &center_count);
    std::cout << "Detected " << center_count << " circles." << std::endl;

    return 0;
}

int main()
{
    // 读取image
    std::string projPath = std::filesystem::path(__FILE__).parent_path().string() + "/";
    std::filesystem::create_directories("./Measure/images");
    std::string imgChessPath = "image/chess.jpg";

    std::string imgCircle55 = "image/5x5circle.png";
    std::string imgCircle77 = "image/7x7circle.png";

    // 测试center获取
    // test_get_img_center(projPath + imgTest1, 1, 10, 200, 1000);
    // test_get_img_center(projPath + imgTest2, 2, 10, 200, 1000);
    if (test_get_img_center(projPath + imgCircle55, 1, 10, 200, 1000) != 0) return 1;
    if (test_get_img_center(projPath + imgCircle77, 2, 10, 200, 1000) != 0) return 1;

    return 0;
}

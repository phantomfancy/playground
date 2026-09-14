// OpenCV_ImageTestCard.h: 标准系统包含文件的包含文件
// 或项目特定的包含文件。

#pragma once

#include <string>
#include <vector>

// TODO: 在此处引用程序需要的其他标头。

int get_img_rect_center(unsigned char *img, int width, int height, int img_index, int *center_point,
                        int buf_size, int side_min, int side_max, int *center_num);

int get_img_circle_center(unsigned char *img, int width, int height, int img_index,
                          int *center_point, int buf_size, int r_min, int r_max, int *center_num);

int get_img_obj_center(unsigned char *img, int width, int height, int img_index, int *center_point,
                       int buf_size, int side_min, int side_max, int *center_num);

// 读取连续的 BGR 三通道数据，缓冲区由调用方持有。
bool load_img_data(const std::string &imagePath, std::vector<unsigned char> &data,
                   int &width, int &height);

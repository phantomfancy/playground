// OpenCV_ImageTestCard.cpp: 定义应用程序的入口点。

#include "OpenCV_ImageTestCard.h"
#include "opencv2/core.hpp"
#include "opencv2/imgcodecs.hpp"
#include "opencv2/imgproc.hpp"
#include <cmath>
#include <numeric>
#include <string>
#include <vector>

const bool DEBUG_IMAGE = true;

using namespace cv;
using namespace std;

// 检查是否是矩形
bool isSquare(const vector<Point> &approx, double tolerance = 0.5)
{
    if (approx.size() != 4) return false;
    if (!isContourConvex(approx)) return false;

    // 计算四条边的长度
    vector<double> lengths;
    for (int i = 0; i < 4; ++i)
    {
        double dx = approx[i].x - approx[(i + 1) % 4].x;
        double dy = approx[i].y - approx[(i + 1) % 4].y;
        lengths.push_back(sqrt(dx * dx + dy * dy));
    }

    // 检查每条边与平均值的偏差是否在容差范围内
    double avg_len = (lengths[0] + lengths[1] + lengths[2] + lengths[3]) / 4.0;

    for (double len : lengths)
    {
        if (abs(len - avg_len) > avg_len * tolerance)
        {
            return false;
        }
    }

    // 可选：检查角度是否接近 90°（通过向量点积）
    for (int i = 0; i < 4; ++i)
    {
        Point p1 = approx[i];
        Point p2 = approx[(i + 1) % 4];
        Point p3 = approx[(i + 2) % 4];

        double dx1 = p1.x - p2.x;
        double dy1 = p1.y - p2.y;
        double dx2 = p3.x - p2.x;
        double dy2 = p3.y - p2.y;

        double dot = dx1 * dx2 + dy1 * dy2;
        double norm1 = sqrt(dx1 * dx1 + dy1 * dy1);
        double norm2 = sqrt(dx2 * dx2 + dy2 * dy2);

        if (norm1 < 1e-6 || norm2 < 1e-6) continue;
        double cos_angle = abs(dot) / (norm1 * norm2); // 点积应接近 0

        if (cos_angle > 0.2) // 允许 ±11.5° 偏差(cos(78.5°)≈0.2)
        {
            return false;
        }
    }

    return true;
}

int get_img_rect_center(unsigned char *img, int width, int height, int img_index, int *center_point,
                        int buf_size, int side_min, int side_max, int *center_num)
{
    *center_num = 0;
    cv::Mat src = cv::Mat(height, width, CV_8UC3, img);
    std::string save_name = "";

    // 转灰度图
    Mat gray;
    Mat src_clone = src.clone();
    cvtColor(src_clone, gray, COLOR_BGR2GRAY);

    Mat Thres;
    GaussianBlur(gray, Thres, Size(5, 5), 0.0);
    threshold(Thres, Thres, 0, 255, THRESH_BINARY_INV | THRESH_OTSU);

    // 轮廓发现
    vector<Vec4i> her;
    vector<vector<Point>> contours;
    // RETR_EXTERNAL	只检测最外层轮廓（忽略所有内部孔洞
    // RETR_LIST	检测所有轮廓，不建立层级关系（所有轮廓平级）
    // RETR_CCOMP	检测所有轮廓，并分为两层：外层（父）和内层（子 / 孔洞）
    // RETR_TREE	检测所有轮廓，并构建完整的嵌套层级树（最完整）
    findContours(Thres.clone(), contours, her, RETR_TREE, CHAIN_APPROX_SIMPLE);
    Mat resultImage = Mat::zeros(gray.size(), CV_8UC3);

    int count = 0;
    double area = 0.0;
    for (size_t i = 0; i < contours.size(); i++)
    {
        if (contours[i].size() < 3) continue;
        double area = contourArea(contours[i], false);
        // 根据面积及纵横比过滤轮廓
        if ((area > (side_min * side_min)) && (area < (side_max * side_max)))
        {
            // 多边形近似
            vector<Point> approx;
            double epsilon = 0.02 * arcLength(contours[i], true);
            if (epsilon <= 0) continue;
            approxPolyDP(contours[i], approx, epsilon, true);
            // 判断是否为正方形
            if (isSquare(approx))
            {
                // 方法1：用矩计算中心
                Moments M = moments(approx);
                Point2f center((float)(M.m10 / M.m00), (float)(M.m01 / M.m00));

                //// 方法2（备选）：外接矩形中心
                // Rect r = boundingRect(approx);
                // Point2f center(r.x + r.width/2.0, r.y + r.height/2.0);
                if (count < buf_size / 2)
                {
                    center_point[(count) * 2] = (int)(center.x * 100);
                    center_point[(count) * 2 + 1] = (int)(center.y * 100);
                    count++;
                }

                // 绘制正方形和中心
                drawContours(resultImage, vector<vector<Point>>{approx}, -1, Scalar(0, 255, 0), 2);
                circle(resultImage, center, 3, Scalar(0, 0, 255), -1); // 红色圆点
            }
        }
    }

    *center_num = count;

    if (count)
    {
        std::string save_name =
            string("./Measure/images/") + "gray" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, gray);

        save_name = string("./Measure/images/") + "thres" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, Thres);

        save_name = string("./Measure/images/") + "detect" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, resultImage);
    }

    return 0;
}

int isCircle(const vector<Point> &approx, double tolerance = 0.2)
{
    // 计算质心，注意避免除以零
    Moments M = moments(approx);
    {
        if (M.m00 == 0) return -1;
    }
    Point2f center((float)(M.m10 / M.m00), (float)(M.m01 / M.m00));
    // 计算所有点到质心的距离
    vector<double> distances;
    for (const Point &p : approx)
    {
        double dx = p.x - center.x;
        double dy = p.y - center.y;
        distances.push_back(sqrt(dx * dx + dy * dy));
    }

    // 检查每个距离与平均半径的偏差是否在容差范围内
    double sum_dist = std::accumulate(distances.begin(), distances.end(), 0.0);
    double avg_radius = sum_dist / distances.size();
    for (double dist : distances)
    {
        if (abs(dist - avg_radius) > avg_radius * tolerance)
        {
            return -1; // 不是圆形
        }
    }
    return 0; // 是圆形
}

int get_img_circle_center(unsigned char *img, int width, int height, int img_index,
                          int *center_point, int buf_size, int r_min, int r_max, int *center_num)
{
    *center_num = 0;
    cv::Mat src = cv::Mat(height, width, CV_8UC3, img);
    Mat gray;
    Mat src_clone = src.clone();
    cvtColor(src_clone, gray, COLOR_BGR2GRAY);

    Mat Thres;
    GaussianBlur(gray, Thres, Size(5, 5), 0.0);
    threshold(Thres, Thres, 0, 255, THRESH_BINARY_INV | THRESH_OTSU);

    vector<Vec4i> her;
    vector<vector<Point>> contours;
    findContours(Thres.clone(), contours, her, RETR_TREE, CHAIN_APPROX_SIMPLE);
    Mat resultImage = Mat::zeros(gray.size(), CV_8UC3);

    int count = 0;
    for (size_t i = 0; i < contours.size(); ++i)
    {
        // if (contours[i].size() < 5) continue; // 设置识别个数的阈值
        double epsilon = 0.02 * arcLength(contours[i], true);
        if (epsilon <= 0) continue;
        vector<Point> approx;
        approxPolyDP(contours[i], approx, epsilon, true);
        // if (approx.size() < 5) continue; // 设置识别个数的阈值

        Point2f center;
        float radius = 0.0f;
        minEnclosingCircle(contours[i], center, radius);
        if (radius < r_min || radius > r_max) continue;

        if (isCircle(approx) == 0 && count < buf_size / 2)
        {
            center_point[count * 2] = static_cast<int>(center.x * 100);
            center_point[count * 2 + 1] = static_cast<int>(center.y * 100);
            ++count;
            circle(resultImage, center, static_cast<int>(radius), Scalar(255, 0, 0), 2);
            circle(resultImage, center, 3, Scalar(0, 255, 255), -1);
        }
    }

    *center_num = count;

    if (DEBUG_IMAGE && count > 0)
    {
        std::string save_name =
            string("./Measure/images/") + "circle_gray" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, gray);

        save_name =
            string("./Measure/images/") + "circle_thres" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, Thres);

        save_name =
            string("./Measure/images/") + "circle_detect" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, resultImage);
    }

    return 0;
}

int get_img_obj_center(unsigned char *img, int width, int height, int img_index, int *center_point,
                       int buf_size, int side_min, int side_max, int *center_num)
{
    *center_num = 0;
    cv::Mat src = cv::Mat(height, width, CV_8UC3, img);
    Mat gray;
    Mat src_clone = src.clone();
    cvtColor(src_clone, gray, COLOR_BGR2GRAY);

    Mat blurred;
    morphologyEx(gray, blurred, MORPH_CLOSE, getStructuringElement(MORPH_RECT, Size(5, 5)));
    GaussianBlur(blurred, blurred, Size(5, 5), 1.5);

    Mat edges;
    Canny(blurred, edges, 50, 150);

    vector<Vec4i> hier;
    vector<vector<Point>> contours;
    findContours(edges.clone(), contours, hier, RETR_TREE, CHAIN_APPROX_SIMPLE);
    Mat resultImage = Mat::zeros(gray.size(), CV_8UC3);

    int count = 0;
    for (size_t i = 0; i < contours.size(); ++i)
    {
        double area = contourArea(contours[i]);
        if (area < (side_min * side_min) || area > (side_max * side_max)) continue;

        Moments M = moments(contours[i]);
        if (M.m00 == 0) continue;
        Point2f center((float)(M.m10 / M.m00), (float)(M.m01 / M.m00));

        if (count < buf_size / 2)
        {
            center_point[count * 2] = static_cast<int>(center.x * 100);
            center_point[count * 2 + 1] = static_cast<int>(center.y * 100);
            ++count;
        }

        drawContours(resultImage, contours, static_cast<int>(i), Scalar(0, 255, 255), 2);
        circle(resultImage, center, 3, Scalar(0, 0, 255), -1);
    }

    *center_num = count;

    if (DEBUG_IMAGE && count > 0)
    {
        std::string save_name =
            string("./Measure/images/") + "obj_gray" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, gray);

        save_name = string("./Measure/images/") + "obj_edges" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, edges);

        save_name = string("./Measure/images/") + "obj_detect" + std::to_string(img_index) + ".bmp";
        imwrite(save_name, resultImage);
    }

    return 0;
}

bool load_img_data(const std::string &imagePath, std::vector<unsigned char> &data,
                   int &width, int &height)
{
    data.clear();
    width = height = 0;
    cv::Mat image = cv::imread(imagePath, cv::IMREAD_COLOR);
    if (image.empty()) return false;
    if (!image.isContinuous()) image = image.clone();
    data.assign(image.data, image.data + image.total() * image.elemSize());
    width = image.cols;
    height = image.rows;
    return true;
}

function float_val = q16_16_to_float(q16_16_val)
% Q16_16_TO_FLOAT 将Q16.16格式定点数转换为浮点数
%
% 输入参数:
%   q16_16_val - 32位有符号整数，表示Q16.16格式定点数
%                高16位为整数部分，低16位为小数部分
%
% 输出参数:
%   float_val - 对应的浮点数值
%
% Q16.16格式说明:
%   - 32位有符号定点数格式
%   - 前16位表示整数部分（相当于int16）
%   - 后16位表示小数部分（相当于uint16）
%   - 小数部分范围：0 到 (2^16-1)/2^16 ≈ 0.999985

    % 确保输入为32位有符号整数
    if q16_16_val > 2147483647
        q16_16_val = q16_16_val - 4294967296;  % 处理无符号到有符号的转换
    end
    
    % 提取整数部分（高16位）
    integer_part = bitshift(q16_16_val, -16);
    
    % 提取小数部分（低16位）
    decimal_part = bitand(q16_16_val, 65535);  % 0xFFFF = 65535
    
    % 将小数部分转换为0~1之间的浮点数
    fractional = double(decimal_part) / 65536.0;  % 2^16 = 65536
    
    % 合并整数部分和小数部分
    float_val = double(integer_part) + fractional;
end
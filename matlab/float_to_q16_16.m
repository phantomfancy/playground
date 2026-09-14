function q16_16_val = float_to_q16_16(float_val)
% FLOAT_TO_Q16_16 将浮点数转换为Q16.16格式定点数
%
% 输入参数:
%   float_val - 浮点数值
%
% 输出参数:
%   q16_16_val - 32位有符号整数，表示Q16.16格式定点数
%
% Q16.16格式说明:
%   - 32位有符号定点数格式
%   - 前16位表示整数部分（范围：-32768 到 32767）
%   - 后16位表示小数部分（精度：1/65536 ≈ 0.0000153）
%   - 总范围：-32768.0 到 32767.999985

    % 饱和处理 - 限制在Q16.16的表示范围内
    if float_val >= 32767.9999847
        q16_16_val = int32(2147483647);  % 0x7FFFFFFF
        return;
    end
    if float_val <= -32768.0
        q16_16_val = int32(-2147483648); % 0x80000000
        return;
    end
    
    % 提取整数部分
    integer_part = int16(fix(float_val));  % 使用fix()截断小数部分
    
    % 计算小数部分
    fractional = float_val - double(integer_part);
    
    % 处理负数的小数部分
    if integer_part < 0 && fractional < 0
        % 当整数部分为负且小数部分为负时，需要调整表示
        integer_part = integer_part - 1;
        decimal_part = uint16(65535 - round(-fractional * 65536.0));
    else
        % 将小数部分转换为16位无符号整数
        decimal_part = uint16(round(abs(fractional) * 65536.0));
    end
    
    % 合并整数部分和小数部分
    % 整数部分左移16位，然后与小数部分按位或
    q16_16_val = int32(bitshift(int32(integer_part), 16)) + int32(decimal_part);
end
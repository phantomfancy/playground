LPFDesignButter_src;
LPFdrawopenloop;

% 进行频率域滤波（时域卷积等于频域相乘）
% 将 dB 转换为线性幅值
gain_linear = 10.^(gain / 20);
% 将相位 deg 转换为 rad
phase_rad = deg2rad(phase);
% 组合复数形式的频率响应 H_system(ω)
H_system = gain_linear .* exp(1j * phase_rad);
% ===== 2. 计算滤波器的频率响应 H_filter(ω) =====
% 在相同的频率点计算滤波器的频率响应
[H_filter, w_filter] = freqz(b, a, freq, f_s); % w_filter 的单位是 rad/sample
% ===== 3. 频域相乘（等效于滤波） =====
H_filtered = H_system .* H_filter;
% ===== 4. 提取滤波后的幅频和相频响应 =====
% 幅值（线性 → dB）
gain_filtered_dB = 20 * log10(abs(H_filtered));
% 相位（rad → deg）
phase_filtered_deg = rad2deg(angle(H_filtered));
% ===== 5. 绘制结果 =====
% 绘制滤波后数据
hold on;
% 左侧Y轴：Gain（蓝色实线）
yyaxis left; 
plot(x, gain_filtered_dB, 'b-.', 'LineWidth', 1); 
% 右侧Y轴：Phase（红色虚线）
yyaxis right; 
plot(x, phase_filtered_deg, 'g-.', 'LineWidth', 1); 
title(sprintf('FRA开环响应曲线, butterworth n=%d, W_n=%f',n,wn));
legend('Gain','Phase','Gain filtered','Phase filtered', ...
    'Location', 'best'); % 图例顺序与绘图顺序一致
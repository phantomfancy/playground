data = importdata("FRA曲线.csv").data;
freq = data(:, 2);        % 第2列作为X轴数据
phase = data(:, 4);   % 第4列作为右侧Y轴数据（Phase）
gain = data(:, 5);    % 第5列作为左侧Y轴数据（Gain）

figure;
% 左侧Y轴：Gain（蓝色实线）
yyaxis left; 
plot(freq, gain, 'b-', 'LineWidth', 1); 
ylabel('Gain (dB)'); 

% 右侧Y轴：Phase（红色虚线）
yyaxis right; 
plot(freq, phase, 'r--', 'LineWidth', 1); 
ylabel('Phase (deg)'); 
ylim([-180 180]);
% 设置X轴为对数坐标
set(gca, 'XScale', 'log');          % 修正：作用于坐标轴对象 [6,7](@ref)
xlabel('Frequency (Hz)');           % 建议明确单位

% 其他设置
title('FRA开环响应曲线');
legend('Gain','Phase', 'Location', 'best'); % 图例顺序与绘图顺序一致
grid on;

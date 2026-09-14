%%
% 生成带有"wn","wn_q16"和"tan_pi2"共三列的数据tan_pi2_data.csv。
% 第一列wn = cat(2,0:0.01:0.99, 0.999,0.9999)；
% 第二列wn_q16=float_to_q16_16(wn)；
% 第三列tan_pi2=tan_pi2(wn)。

% 生成数据矩阵
wn = [0:0.01:0.99, 0.999, 0.9999]';  % 第一列数据
wn_q16 = arrayfun(@float_to_q16_16, wn);  % 第二列转换
tan_pi2 = arrayfun(@tan_pi2, wn);  % 第三列计算

% 合并数据并写入CSV
data_table = table(wn, wn_q16, tan_pi2, ...
    'VariableNames', {'wn', 'wn_q16', 'tan_pi2'});
writetable(data_table, 'tan_pi2_data.csv');

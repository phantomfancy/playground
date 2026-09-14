%%
% 生成带有"wn","b"和"a"共三列的数据tan_pi2_data.csv。
% [b,a]=butter(n,wn),这里取n=2,不同wn对应不同的数组b和a。

% 生成数据矩阵
wn = [0.01:0.01:0.99, 0.999, 0.9999]';  % 第一列数据
b_cell = cell(length(wn),1);
a_cell = cell(length(wn),1);
for i = 1:length(wn)
    [b_tmp, a_tmp] = butter(3, wn(i));
    b_cell{i} = b_tmp;
    a_cell{i} = a_tmp;
end

% 合并数据并写入CSV
data_table = table(wn, b_cell, a_cell, ...
    'VariableNames', {'wn', 'b', 'a'});
writetable(data_table, 'butter_3.csv');

% 离散滤波器设计和使用
% 设置
options = bodeoptions;
options.FreqUnits = 'Hz'; % or 'rad/second',

% 采样率
f_s = 10000;
T_s = 1/f_s;
f_nyquist = f_s/2;
w_nyquist = 2*pi*f_nyquist;

% 设计巴特沃斯低通滤波器
% 带宽500Hz,通频带f_c,阻带1000Hz
bandwidth = 500;
Q = sqrt(1/2);
f_c = bandwidth*Q;
w_c = 2*pi*f_c;
f_sb = 1000;
w_sb = 2*pi*f_sb;
% 生成滤波器
[n,wn] = buttord(f_c/f_nyquist,f_sb/f_nyquist,1,20);
[b,a] = butter(n,wn,"low");
% lpf_butter_design = tf(b,a,T_s); % 离散滤波器
[z,p,k] = butter(n,wn,"low");
lpf_butter_design = zpk(z,p,k,T_s);

%%
% $H\left(z\right)=\frac{Y\left(z\right)}{X\left(z\right)}=\frac{0.002551+0.007654z^{-1}+0.007654z^{-2}+0.002551z^{-3}}{1-2.402z^{-1}+1.97z^{-2}-0.5477z^{-3}}$ 

%%
% $\rightarrow y(n)=0.002551x(n)+0.007654x(n-1)+0.007654x(n-2)+0.002551x(n-3)+2.402y(n-1)+1.97y(n-2)-0.5477y(n-3)$ 

% 显示滤波器bode图
% sos = tf2sos(b,a); %使用tf2sos将高阶滤波器分解成1阶和2阶滤波器的乘积，以便实现
sos = zp2sos(z,p,k);
figure;
freqz(sos,512,10000);
title(sprintf('n = %d Butterworth digital Lowpass Filter',n));
grid on;

% % 使用开环曲线测试低通滤波效果
% data = importdata("FRA曲线.csv").data;
% gain = data(:, 4);   % 第4列作为（Gain）
% phase = data(:, 5);    % 第5列作为（Phase）

%% 常见模拟低通滤波器测试
% 绘制了典型低通滤波器的bode图;
% 比较了他们在500Hz后的增益和相位;
% 在同样带宽下比较增益和相位；

%% 低通滤波器
% 设置
options = bodeoptions;
options.FreqUnits = 'Hz'; % or 'rad/second',

K = 1;      % 增益
f_c = 500;  % 截止频率
% w_c = 2*pi*f_c;

%subplot布局参数
m = 2;  
n = 3;
sgtitle("低通滤波器测试");
%% 一阶低通
% 一阶RC
% w_c=1/RC;
% H(s)=K * 1/(RCs+1)=K * 1/(s/(w_c)+1)
% |H(jw)|=K * 1/sqrt(1+(w/(w_c))^2)
% ∠H(jw)=-arctan(w/w_c)
bandwidth = 500;
f_c = bandwidth;
w_c = 2*pi*f_c;

lpf_RC_fir_order = tf(K*w_c, [1 w_c]);

subplot(m,n,1);
margin(lpf_RC_fir_order,options);
title("1阶RC");
fprintf("filter:1阶RC\n");
fprintf("f_c=%f\n",f_c);
fprintf("bandwidth=%f\n",bandwidth);
fprintf("K=%f\n",K);
grid on;

%% 二阶低通
% 2阶RC

% 2阶Sallen-Key, 可采用巴特沃斯/切比雪夫配置
% w_c=1/sqrt(R1*R2*C1*C2)
% H(s)=K * w_c^2/(s^2+(w_c/Q)s+w_c^2)
% Q=sqrt(R1*R2*C1*C2)/(C1*(R1+R2)+R1*C2*(1-K))
R1 = 22e3;
R2 = 22e3;
C1 = 10e-9;
C2 = 10e-9;
Q=sqrt(R1*R2*C1*C2)/(C1*(R1+R2)+R1*C2*(1-K)); %Q=0.5,截止频率和特征频率不同
bandwidth = 500;
f_c = bandwidth * Q;
w_c = 2*pi*w_c;

lpf_SK_sec_order = tf(K*(w_c^2), [1 w_c/Q w_c^2]);

subplot(m,n,2);
margin(lpf_SK_sec_order,options);
title("2阶SK");
fprintf("filter:2阶SK\n");
fprintf("f_c=%f\n",f_c);
fprintf("bandwidth=%f\n",bandwidth);
fprintf("K=%f\n",K);
grid on;

% 2阶/n阶巴特沃斯
% Q=0.707
% Bandwidth=f_c/Q=1.414f_c
N = 2;
bandwidth = 500;
Q = 1/sqrt(2);
f_c = bandwidth * Q;
w_c =2*pi*f_c;

[Y,X] = butter(N,w_c,'low','s');
lpf_butter_n_order = tf(Y,X);

subplot(m,n,3);
margin(lpf_butter_n_order,options);
title(sprintf("%d阶巴特沃斯",N));
fprintf("filter:%d阶巴特沃斯\n",N);
fprintf("f_c=%f\n",f_c);
fprintf("bandwidth=%f\n",bandwidth);
fprintf("K=%f\n",K);
grid on;

% 2阶切比雪夫
subplot(m,n,4);
margin(lpf_butter_n_order,options);
grid on;
% cheb2ap()

% 2阶/n阶贝塞尔
N = 2;
bandwidth = 500;
f_c = 500;
w_c = 2*pi*f_c;

[b, a] = besself(2,w_c);
lpf_bessel_sec_order = tf(b,a);

subplot(m,n,5);
margin(lpf_bessel_sec_order,options);
title(sprintf("%d阶贝塞尔",N));
fprintf("filter:%d阶贝塞尔\n",N);
fprintf("f_c=%f\n",f_c);
fprintf("bandwidth=%f\n",bandwidth);
fprintf("K=%f\n",K);
grid on;

% 3阶巴特沃斯

% 
figure;
sos_2butter = tf2sos(Y,X);
freqz(sos_2butter,512,10000);
%end

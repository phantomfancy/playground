N = 2;
bandwidth = 500;
Q = 1/sqrt(2);
f_c = bandwidth * Q;
w_c =2*pi*f_c;

[b, a] = butter(N,w_c,'low','s');
lpf_butter_n_order = tf(b,a,1e-4);

figure;
margin(lpf_butter_n_order,options);
title(sprintf("%d阶巴特沃斯",N));
fprintf("filter:%d阶巴特沃斯\n",N);
fprintf("f_c=%f\n",f_c);
fprintf("bandwidth=%f\n",bandwidth);
fprintf("K=%f\n",K);
grid on;
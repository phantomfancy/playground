% clc;clear;
close all;
% MCK System Modelling
k_oisx=0.557;
c_oisx=0.01;
m=4.33e-6;c=1;k=k_oisx;
mckSys = tf(1,[m c k]);
t = 0:0.1:20;
figure;
subplot(1,2,1);
xlabel('Time (s)');ylabel('Amplitude');
title('Sin Response of MCK System');
lsim(mckSys,sin(0.5*pi*t),t);grid on;hold on;
subplot(1,2,2);
title('FRA Response of MCK System');
margin(mckSys);grid on;hold on;

% simple controller design: PID
Kp=18;Ki=72;Kd=1/54;
pidSys = pid(Kp,Ki,Kd);
closedLoopSys = feedback(pidSys*mckSys,1);
figure;
subplot(1,2,1);
step(closedLoopSys);grid on;
title('Step Response of Closed-Loop System');
subplot(1,2,2);
title('FRA Response of Closed-Loop System');
margin(closedLoopSys);grid on;hold on;
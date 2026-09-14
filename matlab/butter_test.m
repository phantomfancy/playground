function ret = butter_test(Wn)
if(Wn<0||Wn>1)
    ret = -1;
    return;
end
[b2,a2] = butter(2,Wn,'low');
[b3,a3] = butter(3,Wn,'low');
disp(tan_pi2(Wn))
b2
a2
b3
a3
ret=0;
% disp(b2);
% disp(a2);
% disp(b3);
% disp(a3);
end
function ret = tan_pi2(Wn)
if(Wn<0||Wn>1)
    ret=-1;
    return;
end
ret = tan(pi/2*Wn);
% disp(ret)
% disp(Wn * (1 + (Wn^2) * (1/3 + (Wn^2) * (2/15)) ) )
return;
end
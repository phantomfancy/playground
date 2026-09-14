function [b,a] = butter_measure(order,f)
    [b,a] = butter(order,f/5000);
    disp(b);
    disp(a);
end


function [b,a] = lpf_butter_wrapper(n, Wn)
    [b,a] = butter(n, Wn);
end
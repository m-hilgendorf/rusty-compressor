% Created by Michael Hilgendorf <mike@hilgendorf.audio> 
% Licensed under GNU GPL
% 
% This script generates the plots for the blog post and test signals for the 
% Rust code 
close all
clear all

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Model: First Order Average
function y = fo_avg (x, time_ms, sample_rate)
    tau = 1 - exp (-2200 / (time_ms * sample_rate)); 
    num = tau;
    den = [1, -(1-tau)];
    y = filter (num, den, x);
end

sample_rate = 48000; 
N = floor (sample_rate * 20 * 0.001 * 0.5);
x = [ones(1, N), zeros(1, N)]; % create the step function 
Tc = 2; % ms
y = fo_avg (x, Tc, sample_rate);

t = 1000.0 * (0:length(x)-1) / sample_rate;

figure
plot(t, x, t, y,
    [Tc, Tc], [-.1,.9], '--sk',    % this stuff is just to make the plot prettier
    [Tc, Tc] + 10, [-.2, .1], '--sk'
    )
axis([0,max(t),-.1,1.1])
set(gca,'ytick',-1:.1:1)
set(gca,'xtick',0:2:20)
xlabel('Time (ms)')
ylabel('Amplitude')
legend('Input (step)', 'Output (avg)')
title("First Order Average")
grid on

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Model: AR Average
% Model: AR Average
function y = ar_avg (x, att_ms, rel_ms, sample_rate, init) 
    at = 1 - exp(-2200 / (att_ms * sample_rate)); 
    rt = 1 - exp(-2200 / (rel_ms * sample_rate));

    y = zeros(size(x));
    yn_1 = init;
    
    for n = 1:length(x)
        if x(n) > yn_1
            tau = at;
        else 
            tau = rt;
        end

        yn_1= tau * x(n) + (1 - tau) * yn_1 ;
        y(n) = yn_1;
    end
end

att = 2; %ms
rel = 6; %ms
y = ar_avg (x, att, rel, sample_rate, 0.0);

figure
plot(t, x, t, y,
    [att, att], [-.2,.9], '--sk',    % this stuff is just to make the plot prettier
    [rel, rel] + 10, [-.2, .1], '--sk'
    )
axis([0,max(t),-.1,1.1])
set(gca,'ytick',-1:.1:1)
set(gca,'xtick',0:2:20)
xlabel('Time (ms)')
ylabel('Amplitude')
legend('Input (step)', 'Output (avg)')
title('AR Average')
grid on
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Model: Peak follower 
% create an envelope to follow 
times = [10.0, 20.0, 30.0];
L     = floor (0.001 * times * sample_rate);                                             
env   = [0.25 * ones(1, L(1)), ones(1, L(2)), 0.5 * ones(1, L(3))]; 
t     = (0:length(env)-1) / length(env);

% input is a sine wave with an interesting envelope
x = env .* sin (20.0 * pi * t);
y = ar_avg (
    abs(x),     % for a peak follower, we take the absolute value   
    0.01,       % we want a really quick, almost zero attack
    30,         % this could be longer, and you can choose whatever works for your application
    sample_rate, 
    0.0         % filter state initialized to zero
);

t = 1000.0 * t;
figure
plot(t,x,t,y, 
     t,env,'--k',
     t,-env,'--k'   
)
set(gca,'ytick',-1:.1:1)
xlabel('Time (ms)')
ylabel('Amplitude')
legend('Signal', 'Peaks', 'Envelope')
title('Peak Follower')
grid on

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Model: Putting it all together... 
function y = gain_calc (x, threshold, ratio) 
    dB   = 20.0 * log10(abs(x));
    gain = min (0, (1 - 1/ratio) * (threshold - dB));
    y    = 10.^(gain / 20.0);
end 

function [output, peaks, gain, smoothed] = compress (
    inpt, 
    threshold, 
    ratio, 
    attack, 
    release,
    makeup, 
    sample_rate )

    peaks    = ar_avg (abs(inpt), 0.01, 10.0, sample_rate, 0.0); % envelope follower 
    gain     = gain_calc (peaks, threshold, ratio);              % gain calculation  
    smoothed = ar_avg (gain, release, attack, sample_rate,1.0);  % gain smoothing, notice attack/release parameters are swapped
    output   = 10.^(makeup /20) .* (smoothed .* inpt);           % makeup gain
end

sample_rate = 48000; % Hz 
threshold   = -6.0;  % dB
ratio       = 10.0; 
attack      = 10.0;  % ms
release     = 25.0;  % ms
makeup      = 0.0;   % dB

% create our test signal
times = [50.0, 50.0, 100.0];
a = [0.25, 1.0, 0.5];
L = floor (0.001 * times * sample_rate);
env =  [a(1) * ones(1, L(1)), a(2) * ones(1, L(2)), a(3) * ones(1, L(3))];
n = 0:length(env)-1;

%inpt = env .* (rand(size(env)) * 2.0 - 1.0);
inpt = env.*sin(100.0 * pi * n / length(n));
[outp, peaks, gain, smoothed] = compress (
    inpt, 
    threshold, 
    ratio, 
    attack, 
    release, 
    makeup, 
    sample_rate
);

t = 1000.0 * (0:length(inpt)-1) / sample_rate;

figure
subplot(2,2,1)
plot(t,inpt,t, env,'Color',[0.8500 0.3250 0.0980],t,-env,'Color',[0.8500 0.3250 0.0980])
xlabel('Time(ms)')
axis([0,max(t),-1.1,1.1])
set(gca, 'ytick', -1:0.5:1);
title('Input')
legend({'Input Signal', 'Envelope'}, 'Location', 'northeast')
grid on

subplot(2,2,3)
plot(t,outp,t,env.*smoothed, 'Color',[0.8500 0.3250 0.0980],t,-env.*smoothed,'Color',[0.8500 0.3250 0.0980])
xlabel('Time(ms)')
axis([0,max(t),-1.1,1.1])
set(gca, 'ytick', -1:0.5:1);
title('Output')
legend('Output Signal', 'Envelope')
grid on

subplot(2,2,2)
plot(t, peaks, t, env, '--k', t, -env,'--k');
xlabel('Time(ms)')
axis([0,max(t),-1.1,1.1])
set(gca, 'ytick', -1:0.5:1)
legend('Detected Peaks','Signal Envelope')
title('Peak Detector')
grid on

subplot(2,2,4)
plot(t, gain, t, smoothed)
xlabel('Time(ms)')
axis([0,max(t),-.1,1.1])
set(gca, 'ytick', -1:0.5:1)
legend({'Calcuated','Smoothed'}, 'Location','northeast')
title('Gain Reduction')
grid on

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Model: generating reference data
sample_rate = 48000; % Hz 
omega    = (2.0 * pi * 2000.0 / 48000.0); % 20Hz at Fs = 48kHz
sine20Hz = sin (omega * (0:511));
case0    = [0.01 * ones(1,256), ones(1,256)].*sine20Hz;
case1    = [ones(1,256), 0.01*ones(1,256)] .*sine20Hz;

threshold   = -6.0;  % dB
ratio       = 10.0; 
attack      = 1.3;  % 64 samples at 48k 
release     = 2.6;  % 128 samples at 48k
makeup      = 0.0;  % dB

filename = "test_signals.rs";
fid = fopen (filename, "w");
inpt = [case0; case1];
fprintf(fid, '!#[allow(dead_code)]\n');
for n = 1:min(size(inpt))
    [outp, peaks, gain, smoothed] = compress (
      inpt(n,:), 
      threshold, 
      ratio, 
      attack, 
      release, 
      makeup, 
      sample_rate
    );

    fprintf(fid, 'pub static CASE_%d_INPUT : [f32; %d] = [', n, length(inpt(n,:)));
    fprintf(fid, '%f,', inpt(n,:));
    fprintf(fid, '];\n');

    fprintf(fid, 'pub static CASE_%d_PEAK: [f32; %d] = [', n, length(inpt(n,:)));
    fprintf(fid, '%f,', peaks);
    fprintf(fid, '];\n');

    fprintf(fid, 'pub static CASE_%d_GAIN : [f32; %d] = [', n, length(inpt(n,:)));
    fprintf(fid, '%f,', gain);
    fprintf(fid, '];\n');

    fprintf(fid, 'pub static CASE_%d_SMOOTHED : [f32; %d] = [',n, length(inpt(n,:)));
    fprintf(fid, '%f,', smoothed);
    fprintf(fid, '];\n');

    fprintf(fid, 'pub static CASE_%d_OUTP : [f32; %d] = [',n, length(inpt(n,:)));
    fprintf(fid, '%f,', outp);
    fprintf(fid, '];\n');
end
fclose(fid);
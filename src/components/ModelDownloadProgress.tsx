import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { listen } from '@tauri-apps/api/event';

interface ProgressData {
    status: string;
    message: string;
    filename: string;
    current: number;
    total: number;
    percentage: number;
    formatted: string;
}

// Set to true to show a simulated download progress (for development only)
const DEBUG_MODE = false;

export function ModelDownloadProgress() {
    const [isDownloading, setIsDownloading] = createSignal(DEBUG_MODE);
    const [progress, setProgress] = createSignal<ProgressData>({
        status: 'downloading',
        message: 'Downloading gemma-3-1b-it-Q8_0.gguf - 0.9%',
        filename: 'gemma-3-1b-it-Q8_0.gguf',
        current: DEBUG_MODE ? 9_540_000 : 0,
        total: DEBUG_MODE ? 1_019_770_000 : 0,
        percentage: DEBUG_MODE ? 0.9 : 0,
        formatted: DEBUG_MODE ? '0.9%' : '0%'
    });

    // Debug mode: simulate download progress
    createEffect(() => {
        if (!DEBUG_MODE) return;

        let currentBytes = progress().current;
        const totalBytes = progress().total;
        const intervalId = setInterval(() => {
            // Increment by a random amount between 5-15MB
            const increment = Math.floor(Math.random() * 10_000_000) + 5_000_000;
            currentBytes = Math.min(currentBytes + increment, totalBytes);
            const percentage = (currentBytes / totalBytes) * 100;
            const formattedPercentage = `${percentage.toFixed(1)}%`;

            setProgress({
                status: currentBytes >= totalBytes ? 'completed' : 'downloading',
                message: currentBytes >= totalBytes
                    ? 'Model downloaded successfully!'
                    : `Downloading gemma-3-1b-it-Q8_0.gguf - ${formattedPercentage}`,
                filename: 'gemma-3-1b-it-Q8_0.gguf',
                current: currentBytes,
                total: totalBytes,
                percentage: percentage,
                formatted: formattedPercentage
            });

            if (currentBytes >= totalBytes) {
                clearInterval(intervalId);
                // Keep visible for a few seconds after completion
                setTimeout(() => {
                    if (DEBUG_MODE) {
                        // Reset and start again for demo purposes
                        currentBytes = 0;
                        setProgress({
                            ...progress(),
                            current: 0,
                            percentage: 0,
                            formatted: '0.0%',
                            status: 'downloading',
                            message: 'Downloading gemma-3-1b-it-Q8_0.gguf - 0.0%'
                        });
                    } else {
                        setIsDownloading(false);
                    }
                }, 3000);
            }
        }, 800); // Update every 800ms

        onCleanup(() => clearInterval(intervalId));
    });

    createEffect(() => {
        // Only set up real listeners if not in debug mode
        if (DEBUG_MODE) return;

        // Listen for model download progress events
        const unlisten = listen<ProgressData>('model-download-progress', (event) => {
            setProgress(event.payload);

            // Set downloading state based on status
            if (['initializing', 'downloading', 'downloading_config', 'downloading_model'].includes(event.payload.status)) {
                setIsDownloading(true);
            } else if (['completed', 'success'].includes(event.payload.status)) {
                // Keep progress visible for a moment after completion
                setTimeout(() => setIsDownloading(false), 3000);
            }
        });

        onCleanup(() => {
            // Clean up event listener
            unlisten.then(unlistenFn => unlistenFn());
        });
    });

    // Format bytes to human-readable format
    const formatBytes = (bytes: number) => {
        if (bytes === 0) return '0 Bytes';

        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${sizes[i]}`;
    };

    return (
        <Show when={isDownloading()}>
            <div class="bg-[#292929] border-t border-[#444] overflow-hidden">
                <div class="px-3 py-2 bg-[#333] border-b border-[#444]">
                    <div class="flex items-center justify-between">
                        <h3 class="text-sm font-medium text-white truncate">
                            Downloading Model
                        </h3>
                        <span class="text-xs text-white ml-1">
                            {progress().formatted}
                        </span>
                    </div>
                </div>

                <div class="px-3 py-2">
                    {/* Filename on its own line */}
                    <div class="text-xs text-[#ccc] mb-1 font-mono truncate">
                        {progress().filename}
                    </div>



                    <div class="w-full h-1 bg-[#444] rounded overflow-hidden mb-1.5">
                        <div
                            class="h-full bg-blue-500 transition-all duration-300 ease-out"
                            style={{ width: `${progress().percentage}%` }}
                        />
                    </div>

                    {/* Size info on its own line */}
                    <Show when={progress().total > 0}>
                        <div class="text-xs text-[#aaa] mb-1.5 text-right">
                            {formatBytes(progress().current)} / {formatBytes(progress().total)}
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    );
} 
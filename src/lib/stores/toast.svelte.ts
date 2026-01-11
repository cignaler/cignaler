interface ToastMessage {
    id: number;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration: number;
}

let toastId = 0;

export const toastState = $state({
    toasts: [] as ToastMessage[]
});

export function showToast(type: ToastMessage['type'], message: string, duration = 5000) {
    const id = ++toastId;
    toastState.toasts = [...toastState.toasts, { id, type, message, duration }];

    if (duration > 0) {
        setTimeout(() => dismissToast(id), duration);
    }
}

export function dismissToast(id: number) {
    toastState.toasts = toastState.toasts.filter(t => t.id !== id);
}

// Convenience functions
export const toast = {
    success: (msg: string, duration?: number) => showToast('success', msg, duration),
    error: (msg: string, duration?: number) => showToast('error', msg, duration),
    warning: (msg: string, duration?: number) => showToast('warning', msg, duration),
    info: (msg: string, duration?: number) => showToast('info', msg, duration)
};

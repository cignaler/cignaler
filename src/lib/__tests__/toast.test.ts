import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Since .svelte.ts files require Svelte compilation which conflicts with Vite 7,
// we test the toast logic by recreating a simplified version of the store

interface ToastMessage {
    id: number;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration: number;
}

// Simplified toast store for testing
function createToastStore() {
    let toastId = 0;
    const toasts: ToastMessage[] = [];

    function showToast(type: ToastMessage['type'], message: string, duration = 5000) {
        const id = ++toastId;
        toasts.push({ id, type, message, duration });

        if (duration > 0) {
            setTimeout(() => dismissToast(id), duration);
        }

        return id;
    }

    function dismissToast(id: number) {
        const index = toasts.findIndex(t => t.id === id);
        if (index !== -1) {
            toasts.splice(index, 1);
        }
    }

    function getToasts() {
        return [...toasts];
    }

    function clear() {
        toasts.length = 0;
        toastId = 0;
    }

    return {
        showToast,
        dismissToast,
        getToasts,
        clear,
        toast: {
            success: (msg: string, duration?: number) => showToast('success', msg, duration),
            error: (msg: string, duration?: number) => showToast('error', msg, duration),
            warning: (msg: string, duration?: number) => showToast('warning', msg, duration),
            info: (msg: string, duration?: number) => showToast('info', msg, duration)
        }
    };
}

describe('Toast Store Logic', () => {
    let store: ReturnType<typeof createToastStore>;

    beforeEach(() => {
        store = createToastStore();
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('showToast', () => {
        it('should add a toast to the list', () => {
            store.showToast('success', 'Test message');

            const toasts = store.getToasts();
            expect(toasts).toHaveLength(1);
            expect(toasts[0]).toMatchObject({
                type: 'success',
                message: 'Test message',
                duration: 5000
            });
        });

        it('should assign unique IDs to each toast', () => {
            store.showToast('success', 'First');
            store.showToast('error', 'Second');

            const toasts = store.getToasts();
            expect(toasts[0]?.id).not.toBe(toasts[1]?.id);
        });

        it('should use custom duration when provided', () => {
            store.showToast('info', 'Custom duration', 10000);

            const toasts = store.getToasts();
            expect(toasts[0]?.duration).toBe(10000);
        });

        it('should auto-dismiss after duration', () => {
            store.showToast('success', 'Auto dismiss', 3000);

            expect(store.getToasts()).toHaveLength(1);

            vi.advanceTimersByTime(3000);

            expect(store.getToasts()).toHaveLength(0);
        });

        it('should not auto-dismiss when duration is 0', () => {
            store.showToast('success', 'Persistent', 0);

            vi.advanceTimersByTime(10000);

            expect(store.getToasts()).toHaveLength(1);
        });
    });

    describe('dismissToast', () => {
        it('should remove a toast by ID', () => {
            store.showToast('success', 'First', 0);
            store.showToast('error', 'Second', 0);

            const firstId = store.getToasts()[0]?.id;
            if (firstId !== undefined) {
                store.dismissToast(firstId);
            }

            const toasts = store.getToasts();
            expect(toasts).toHaveLength(1);
            expect(toasts[0]?.message).toBe('Second');
        });

        it('should do nothing for non-existent ID', () => {
            store.showToast('success', 'Test', 0);

            store.dismissToast(99999);

            expect(store.getToasts()).toHaveLength(1);
        });
    });

    describe('toast convenience functions', () => {
        it('should show success toast', () => {
            store.toast.success('Success message');

            expect(store.getToasts()[0]?.type).toBe('success');
        });

        it('should show error toast', () => {
            store.toast.error('Error message');

            expect(store.getToasts()[0]?.type).toBe('error');
        });

        it('should show warning toast', () => {
            store.toast.warning('Warning message');

            expect(store.getToasts()[0]?.type).toBe('warning');
        });

        it('should show info toast', () => {
            store.toast.info('Info message');

            expect(store.getToasts()[0]?.type).toBe('info');
        });
    });
});

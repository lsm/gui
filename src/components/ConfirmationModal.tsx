import { Show } from "solid-js";

interface ConfirmationModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationModal(props: ConfirmationModalProps) {
  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm transition-opacity duration-300"
        onClick={(e) => {
          // Close modal if backdrop is clicked
          if (e.target === e.currentTarget) {
            props.onCancel();
          }
        }}
      >
        <div 
          class="bg-bg-secondary rounded-lg shadow-xl p-6 w-full max-w-md mx-4 transform transition-all duration-300 scale-100 opacity-100"
          onClick={(e) => e.stopPropagation()} // Prevent clicks inside modal from closing it
        >
          <h2 class="text-xl font-semibold text-text-primary mb-4">{props.title}</h2>
          <p class="text-text-secondary mb-6">{props.message}</p>
          <div class="flex justify-end space-x-3">
            <button 
              onClick={props.onCancel}
              class="px-4 py-2 rounded bg-gray-600 text-white hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-opacity-50 transition-colors duration-200"
            >
              Cancel
            </button>
            <button 
              onClick={props.onConfirm}
              class="px-4 py-2 rounded bg-red-600 text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-50 transition-colors duration-200"
            >
              Confirm
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
} 
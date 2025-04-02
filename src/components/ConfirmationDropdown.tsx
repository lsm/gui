import { JSX } from "solid-js";

interface ConfirmationDropdownProps {
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationDropdown(props: ConfirmationDropdownProps) {
  return (
    <div 
      class="absolute top-full right-0 mt-1 w-48 bg-bg-secondary rounded-md shadow-lg z-50 border border-border p-2"
      // Prevent clicks inside the dropdown from propagating further (e.g., to a click-outside handler)
      onClick={(e) => e.stopPropagation()} 
    >
      <p class="text-sm text-text-secondary mb-2 px-1">Are you sure?</p>
      <div class="flex justify-between space-x-2">
        <button 
          onClick={props.onCancel}
          class="flex-1 px-2 py-1 text-xs rounded bg-gray-600 text-white hover:bg-gray-700 focus:outline-none focus:ring-1 focus:ring-gray-500 transition-colors duration-150"
        >
          Cancel
        </button>
        <button 
          onClick={props.onConfirm}
          class="flex-1 px-2 py-1 text-xs rounded bg-red-600 text-white hover:bg-red-700 focus:outline-none focus:ring-1 focus:ring-red-500 transition-colors duration-150"
        >
          Confirm
        </button>
      </div>
    </div>
  );
} 
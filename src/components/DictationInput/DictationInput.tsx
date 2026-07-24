import "./DictationInput.css";

interface DictationInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  disabled?: boolean;
}

export default function DictationInput({ value, onChange, onSubmit, disabled }: DictationInputProps) {
  return (
    <div className="dictation-input card">
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            onSubmit();
          }
        }}
        disabled={disabled}
        placeholder="Type what you hear..."
        rows={4}
      />
      <div className="dictation-input__foot">
        <span className="dictation-input__hint">
          <kbd>Ctrl</kbd>/<kbd>Cmd</kbd> + <kbd>Enter</kbd> to check
        </span>
        <button
          type="button"
          className="btn btn-primary"
          onClick={onSubmit}
          disabled={disabled || value.trim().length === 0}
        >
          Check
        </button>
      </div>
    </div>
  );
}

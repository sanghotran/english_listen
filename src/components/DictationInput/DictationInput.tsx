interface DictationInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  disabled?: boolean;
}

export default function DictationInput({ value, onChange, onSubmit, disabled }: DictationInputProps) {
  return (
    <div className="dictation-input">
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
        placeholder="Type what you hear... (Ctrl/Cmd+Enter to check)"
        rows={4}
      />
      <button type="button" onClick={onSubmit} disabled={disabled || value.trim().length === 0}>
        Check
      </button>
    </div>
  );
}

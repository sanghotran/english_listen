import "./Mascot.css";

interface MascotProps {
  size?: number;
  className?: string;
}

/** The app's default mascot: used for page headers and empty states. */
export function MascotBunny({ size = 44, className }: MascotProps) {
  return (
    <svg
      className={`mascot mascot-bunny${className ? ` ${className}` : ""}`}
      width={size}
      height={size}
      viewBox="0 0 100 100"
      aria-hidden="true"
    >
      <path className="mascot-ear" d="M 39 32 C 27 18, 10 22, 12 42 C 13 54, 24 60, 34 53 Z" />
      <path className="mascot-ear" d="M 61 32 C 73 18, 90 22, 88 42 C 87 54, 76 60, 66 53 Z" />
      <ellipse className="mascot-ear-inner" cx="23" cy="39" rx="4.4" ry="10" transform="rotate(-18 23 39)" />
      <ellipse className="mascot-ear-inner" cx="77" cy="39" rx="4.4" ry="10" transform="rotate(18 77 39)" />
      <ellipse className="mascot-body" cx="50" cy="58" rx="38" ry="33" />
      <ellipse className="mascot-highlight" cx="33" cy="40" rx="11" ry="6" transform="rotate(-24 33 40)" />
      <path className="mascot-bow" d="M 50 26 L 39 17 L 39 35 Z" />
      <path className="mascot-bow" d="M 50 26 L 61 17 L 61 35 Z" />
      <circle className="mascot-bow-knot" cx="50" cy="26" r="4.2" />
      <ellipse className="mascot-eye" cx="37" cy="56" rx="4" ry="5.4" />
      <ellipse className="mascot-eye" cx="63" cy="56" rx="4" ry="5.4" />
      <ellipse className="mascot-blush" cx="26" cy="66" rx="8" ry="4.8" />
      <ellipse className="mascot-blush" cx="74" cy="66" rx="8" ry="4.8" />
      <path className="mascot-mouth" d="M 41 68 Q 50 75 59 68" />
    </svg>
  );
}

/** Secondary mascot used for celebratory / encouraging moments (e.g. an empty progress state). */
export function MascotBear({ size = 44, className }: MascotProps) {
  return (
    <svg
      className={`mascot mascot-bear${className ? ` ${className}` : ""}`}
      width={size}
      height={size}
      viewBox="0 0 100 100"
      aria-hidden="true"
    >
      <ellipse className="bear-ear" cx="26" cy="27" rx="11" ry="11" />
      <ellipse className="bear-ear" cx="74" cy="27" rx="11" ry="11" />
      <ellipse className="bear-body" cx="50" cy="57" rx="36" ry="32" />
      <ellipse className="mascot-highlight" cx="34" cy="41" rx="10" ry="6" transform="rotate(-22 34 41)" />
      <path className="bear-eye" d="M 33 54 Q 37.5 58.5 42 54" />
      <path className="bear-eye" d="M 58 54 Q 62.5 58.5 67 54" />
      <ellipse className="bear-nose" cx="50" cy="63" rx="4.4" ry="3.2" />
      <ellipse className="mascot-blush" cx="27" cy="65" rx="6.5" ry="4" />
      <ellipse className="mascot-blush" cx="73" cy="65" rx="6.5" ry="4" />
      <path className="bear-bow" d="M 50 82 L 41 75 L 41 89 Z" />
      <path className="bear-bow" d="M 50 82 L 59 75 L 59 89 Z" />
      <circle className="bear-bow-knot" cx="50" cy="82" r="3.6" />
    </svg>
  );
}

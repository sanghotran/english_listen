import { Component, type ErrorInfo, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ErrorBoundary.css";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/** Catches render errors that would otherwise unmount the whole app to a blank window,
 * shows a fallback message, and forwards the error to the Rust file log (the frontend
 * has no console visible in a release build otherwise). */
export default class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    invoke("log_frontend_error", { message: `${error.stack ?? error.message}\n${info.componentStack}` }).catch(
      () => {},
    );
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return (
      <div className="error-boundary" role="alert">
        <h1>Something went wrong</h1>
        <p>{error.message}</p>
        <p className="error-boundary__hint">Details were written to english-listen.log next to the app.</p>
      </div>
    );
  }
}

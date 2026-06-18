import type { QuickAnswer } from "../types";

interface QuickAnswerBarProps {
  answer: QuickAnswer;
}

export default function QuickAnswerBar({ answer }: QuickAnswerBarProps) {
  return (
    <div className="quick-answer">
      <span className="quick-answer-label">{answer.label}</span>
      <span className="quick-answer-value">{answer.value}</span>
      {answer.hint && (
        <span className="quick-answer-hint">{answer.hint}</span>
      )}
    </div>
  );
}

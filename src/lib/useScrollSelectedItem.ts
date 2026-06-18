import { useEffect, useRef } from "react";

export function useScrollSelectedItem<T extends HTMLElement>(selectedIndex: number) {
  const selectedRef = useRef<T | null>(null);

  useEffect(() => {
    selectedRef.current?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  const setSelectedRef = (el: T | null) => {
    selectedRef.current = el;
  };

  return setSelectedRef;
}

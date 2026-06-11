import { getCurrentWindow } from "@tauri-apps/api/window";

export default function SoftLockPage() {
  return (
    <div className="h-screen flex flex-col items-center justify-center bg-fp-bg text-fp-text">
      <h1 className="text-4xl font-bold mb-4">
        Daily Limit Exceeded
      </h1>

      <p className="text-fp-muted mb-8">
        You have exceeded the allowed usage time.
      </p>

      <button
        onClick={() => {
          console.log("CONTINUE CLICKED");
          getCurrentWindow().close();
        }}
        className="px-5 py-2 rounded-lg bg-fp-accent text-white"
      >
        Continue Anyway
      </button>
    </div>
  );
}

// export default function SoftLockPage() {
//   return (
//     <div
//       style={{
//         width: "100vw",
//         height: "100vh",
//         background: "black",
//         color: "white",
//         display: "flex",
//         flexDirection: "column",
//         justifyContent: "center",
//         alignItems: "center",
//         gap: "20px",
//       }}
//     >
//       <h1>SOFT LOCK TEST</h1>

//       <button
//         onClick={() => {
//           alert("BUTTON CLICKED");
//         }}
//         style={{
//           padding: "20px",
//           cursor: "pointer",
//         }}
//       >
//         TEST BUTTON
//       </button>
//     </div>
//   );
// }

// export default function SoftLockPage() {
//   return (
//     <button
//       onClick={() => alert("CLICKED")}
//       style={{
//         width: "100vw",
//         height: "100vh",
//         fontSize: "40px",
//       }}
//     >
//       CLICK ME
//     </button>
//   );
// }
using System;
using System.Runtime.InteropServices;

namespace VNKey.Windows.Engine
{
    // Đảm bảo kiểu bool được truyền bằng 1 byte chuẩn xác với Rust ABI để tránh crash UI (PInvokeStackImbalance)
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void ToggleCallbackDelegate([MarshalAs(UnmanagedType.I1)] bool isEnabled);

    public class EngineWrapper : IDisposable
    {
        private const string LibName = "vnkey_core.dll";

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_hook_start();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_hook_stop();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_set_mode(byte mode);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_set_vietnamese_mode([MarshalAs(UnmanagedType.I1)] bool enabled);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_set_toggle_callback(IntPtr callbackMethod);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_set_config_json([MarshalAs(UnmanagedType.LPUTF8Str)] string json);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_set_shorthand_json([MarshalAs(UnmanagedType.LPUTF8Str)] string json);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        [return: MarshalAs(UnmanagedType.I1)]
        public static extern bool vnkey_global_process_backspace();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_reset();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_global_load_dictionary([MarshalAs(UnmanagedType.LPUTF8Str)] string path);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr vnkey_global_get_diagnostic_info();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void vnkey_free_string(IntPtr s);

        public EngineWrapper(Models.InputMode mode = Models.InputMode.Telex)
        {
            vnkey_global_set_mode((byte)mode);
        }

        public void SetMode(Models.InputMode mode)
        {
            vnkey_global_set_mode((byte)mode);
        }

        public void SetVietnameseMode(bool enabled)
        {
            vnkey_global_set_vietnamese_mode(enabled);
        }

        public void Reset()
        {
            vnkey_global_reset();
        }

        public string GetDiagnosticInfo()
        {
            IntPtr ptr = vnkey_global_get_diagnostic_info();
            if (ptr == IntPtr.Zero) return string.Empty;

            try
            {
                string? json = Marshal.PtrToStringUTF8(ptr);
                return json ?? string.Empty;
            }
            finally
            {
                vnkey_free_string(ptr);
            }
        }

        public void Dispose()
        {
        }
    }
}

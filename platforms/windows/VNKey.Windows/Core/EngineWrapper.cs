using System;
using System.Runtime.InteropServices;

namespace VNKey.Windows.Core
{
    public enum InputMode : byte
    {
        Telex = 0,
        Vni = 1,
        Viqr = 2,
        TelexEx = 3
    }

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
        public static extern void vnkey_global_load_dictionary([MarshalAs(UnmanagedType.LPUTF8Str)] string path);

        public EngineWrapper(InputMode mode = InputMode.Telex)
        {
            vnkey_global_set_mode((byte)mode);
        }

        public void SetMode(InputMode mode)
        {
            vnkey_global_set_mode((byte)mode);
        }

        public void SetVietnameseMode(bool enabled)
        {
            vnkey_global_set_vietnamese_mode(enabled);
        }

        public void Reset()
        {
        }

        public void Dispose()
        {
        }
    }
}

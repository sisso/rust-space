
// Generated by flapigen. Do not edit.

// This warning occurs, because both Rust and C# have mehtod `ToString()`.
#pragma warning disable CS0114

using System;
using System.Runtime.InteropServices;

namespace ffi_domain_2
{
    internal static class RustString {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void c_string_delete(IntPtr c_char_ptr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* *mut RustString */ IntPtr c_str_u16_to_string(/* *const u16 */ IntPtr c_string_ptr);

        internal static string rust_to_dotnet(/* *const u16 */ IntPtr c_string_ptr)
        {
            var dotnet_str = Marshal.PtrToStringUni(c_string_ptr);
            RustString.c_string_delete(c_string_ptr);
            return dotnet_str;
        }

        internal static /* *mut RustString */ IntPtr dotnet_to_rust(string dotnet_str)
        {
            var c_string_ptr = Marshal.StringToHGlobalUni(dotnet_str);
            var rust_string_ptr = c_str_u16_to_string(c_string_ptr);
            Marshal.FreeHGlobal(c_string_ptr);
            return rust_string_ptr;
        }
    }

    [System.Serializable]
    public class Error : System.Exception
    {
        public Error(string message) : base(message) { }
    }

    
    public enum ObjKind {
        Fleet = 0,Asteroid = 1,Station = 2
    }
    
    public enum EventKind {
        Add = 0,Move = 1,Jump = 2,Dock = 3,Undock = 4
    }
    
    public class SectorData: IDisposable {
        internal IntPtr nativePtr;

        internal SectorData(IntPtr nativePtr) {
            this.nativePtr = nativePtr;
        }

        public void Dispose() {
            DoDispose();
            GC.SuppressFinalize(this);
        }

        private void DoDispose() {
            if (nativePtr != IntPtr.Zero) {
                SectorData_delete(nativePtr);
                nativePtr = IntPtr.Zero;
            }
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void SectorData_delete(IntPtr __this);

        ~SectorData() {
            DoDispose();
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong SectorData_get_id(/* SectorData */ IntPtr __this);

        
        public  ulong GetId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SectorData_get_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SectorData_get_coords(/* SectorData */ IntPtr __this);

        
        public  Tuple<float, float> GetCoords() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SectorData_get_coords(__this_0);
            var __ret_1 = RustTuple2Tfloatfloat.rust_to_dotnet(__ret_0);
            return __ret_1;
        }
} // class

    
    public class JumpData: IDisposable {
        internal IntPtr nativePtr;

        internal JumpData(IntPtr nativePtr) {
            this.nativePtr = nativePtr;
        }

        public void Dispose() {
            DoDispose();
            GC.SuppressFinalize(this);
        }

        private void DoDispose() {
            if (nativePtr != IntPtr.Zero) {
                JumpData_delete(nativePtr);
                nativePtr = IntPtr.Zero;
            }
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void JumpData_delete(IntPtr __this);

        ~JumpData() {
            DoDispose();
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong JumpData_get_id(/* JumpData */ IntPtr __this);

        
        public  ulong GetId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = JumpData_get_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr JumpData_get_coords(/* JumpData */ IntPtr __this);

        
        public  Tuple<float, float> GetCoords() {
            var __this_0 = this.nativePtr;

            var __ret_0 = JumpData_get_coords(__this_0);
            var __ret_1 = RustTuple2Tfloatfloat.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong JumpData_get_sector_id(/* JumpData */ IntPtr __this);

        
        public  ulong GetSectorId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = JumpData_get_sector_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong JumpData_get_to_sector_id(/* JumpData */ IntPtr __this);

        
        public  ulong GetToSectorId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = JumpData_get_to_sector_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr JumpData_get_to_coords(/* JumpData */ IntPtr __this);

        
        public  Tuple<float, float> GetToCoords() {
            var __this_0 = this.nativePtr;

            var __ret_0 = JumpData_get_to_coords(__this_0);
            var __ret_1 = RustTuple2Tfloatfloat.rust_to_dotnet(__ret_0);
            return __ret_1;
        }
} // class

    
    public class ObjData: IDisposable {
        internal IntPtr nativePtr;

        internal ObjData(IntPtr nativePtr) {
            this.nativePtr = nativePtr;
        }

        public void Dispose() {
            DoDispose();
            GC.SuppressFinalize(this);
        }

        private void DoDispose() {
            if (nativePtr != IntPtr.Zero) {
                ObjData_delete(nativePtr);
                nativePtr = IntPtr.Zero;
            }
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void ObjData_delete(IntPtr __this);

        ~ObjData() {
            DoDispose();
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong ObjData_get_id(/* ObjData */ IntPtr __this);

        
        public  ulong GetId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_get_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong ObjData_get_sector_id(/* ObjData */ IntPtr __this);

        
        public  ulong GetSectorId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_get_sector_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr ObjData_get_coords(/* ObjData */ IntPtr __this);

        
        public  Tuple<float, float> GetCoords() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_get_coords(__this_0);
            var __ret_1 = RustTuple2Tfloatfloat.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr ObjData_get_docked_id(/* ObjData */ IntPtr __this);

        
        public  Option<ulong> GetDockedId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_get_docked_id(__this_0);
            var __ret_1 = RustOptionulong.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte ObjData_is_docked(/* ObjData */ IntPtr __this);

        
        public  bool IsDocked() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_is_docked(__this_0);
            var __ret_1 = (__ret_0 != 0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern uint ObjData_get_kind(/* ObjData */ IntPtr __this);

        
        public  ObjKind GetKind() {
            var __this_0 = this.nativePtr;

            var __ret_0 = ObjData_get_kind(__this_0);
            var __ret_1 = (ObjKind)__ret_0;
            return __ret_1;
        }
} // class

    
    public class EventData: IDisposable {
        internal IntPtr nativePtr;

        internal EventData(IntPtr nativePtr) {
            this.nativePtr = nativePtr;
        }

        public void Dispose() {
            DoDispose();
            GC.SuppressFinalize(this);
        }

        private void DoDispose() {
            if (nativePtr != IntPtr.Zero) {
                EventData_delete(nativePtr);
                nativePtr = IntPtr.Zero;
            }
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void EventData_delete(IntPtr __this);

        ~EventData() {
            DoDispose();
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern uint EventData_get_kind(/* EventData */ IntPtr __this);

        
        public  EventKind GetKind() {
            var __this_0 = this.nativePtr;

            var __ret_0 = EventData_get_kind(__this_0);
            var __ret_1 = (EventKind)__ret_0;
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong EventData_get_id(/* EventData */ IntPtr __this);

        
        public  ulong GetId() {
            var __this_0 = this.nativePtr;

            var __ret_0 = EventData_get_id(__this_0);
            var __ret_1 = __ret_0;
            return __ret_1;
        }
} // class

    
    public class SpaceGame: IDisposable {
        internal IntPtr nativePtr;

        internal SpaceGame(IntPtr nativePtr) {
            this.nativePtr = nativePtr;
        }

        public void Dispose() {
            DoDispose();
            GC.SuppressFinalize(this);
        }

        private void DoDispose() {
            if (nativePtr != IntPtr.Zero) {
                SpaceGame_delete(nativePtr);
                nativePtr = IntPtr.Zero;
            }
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void SpaceGame_delete(IntPtr __this);

        ~SpaceGame() {
            DoDispose();
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* SpaceGame */ IntPtr SpaceGame_new(/* Option */ IntPtr rgs);

        
        public  SpaceGame (System.Collections.Generic.List<string> rgs_0) {
            var rgs_1 = RustVecstring.dotnet_to_rust(rgs_0);
            this.nativePtr = SpaceGame_new(rgs_1);
            
            
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SpaceGame_get_sectors(/* SpaceGame */ IntPtr __this);

        
        public  System.Collections.Generic.List<SectorData> GetSectors() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SpaceGame_get_sectors(__this_0);
            var __ret_1 = RustVecSectorData.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SpaceGame_get_jumps(/* SpaceGame */ IntPtr __this);

        
        public  System.Collections.Generic.List<JumpData> GetJumps() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SpaceGame_get_jumps(__this_0);
            var __ret_1 = RustVecJumpData.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SpaceGame_get_fleets(/* SpaceGame */ IntPtr __this);

        
        public  System.Collections.Generic.List<ObjData> GetFleets() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SpaceGame_get_fleets(__this_0);
            var __ret_1 = RustVecObjData.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SpaceGame_get_obj(/* SpaceGame */ IntPtr __this, ulong id);

        
        public  Option<ObjData> GetObj(ulong id_0) {
            var __this_0 = this.nativePtr;
var id_1 = id_0;
            var __ret_0 = SpaceGame_get_obj(__this_0, id_1);
            var __ret_1 = RustOptionObjData.rust_to_dotnet(__ret_0);
            return __ret_1;
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void SpaceGame_update(/* SpaceGame */ IntPtr __this, float delta);

        
        public  void Update(float delta_0) {
            var __this_0 = this.nativePtr;
var delta_1 = delta_0;
            SpaceGame_update(__this_0, delta_1);
            
            
        }

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option */ IntPtr SpaceGame_take_events(/* SpaceGame */ IntPtr __this);

        
        public  System.Collections.Generic.List<EventData> TakeEvents() {
            var __this_0 = this.nativePtr;

            var __ret_0 = SpaceGame_take_events(__this_0);
            var __ret_1 = RustVecEventData.rust_to_dotnet(__ret_0);
            return __ret_1;
        }
} // class

    internal static class RustOptionulong {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustOptionulong_new_none();

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustOptionulong_new_some(ulong value);
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong RustOptionulong_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustOptionulong_is_some(IntPtr optPtr);

        internal static Option<ulong> rust_to_dotnet(IntPtr optPtr)
        {
            if (RustOptionulong_is_some(optPtr) != 0)
            {
                var value_0 = RustOptionulong_take(optPtr);
                var value_1 = value_0;
                return new Option<ulong>(value_1);
            }
            else
            {
                return new Option<ulong>();
            }
        }

        internal static IntPtr dotnet_to_rust(Option<ulong> opt)
        {
            if (opt.IsSome)
            {
                var value_0 = opt.Value;
                return RustOptionulong_new_some(value_0);
            }
            else
            {
                return RustOptionulong_new_none();
            }
        }
    }
    
    internal static class RustTuple2Tfloatfloat {

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Tuple */ IntPtr RustTuple2Tfloatfloat_new(float t_1, float t_2);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern float RustTuple2Tfloatfloat_take_1(IntPtr tuple);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern float RustTuple2Tfloatfloat_take_2(IntPtr tuple);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustTuple2Tfloatfloat_delete(IntPtr tuple);

        internal static Tuple<float, float> rust_to_dotnet(IntPtr rustTuple)
        {
            var t_1_rust = RustTuple2Tfloatfloat_take_1(rustTuple);
            var t_1 = t_1_rust;
            var t_2_rust = RustTuple2Tfloatfloat_take_2(rustTuple);
            var t_2 = t_2_rust;
            var ret = Tuple.Create(t_1, t_2);
            RustTuple2Tfloatfloat_delete(rustTuple);
            return ret;
        }
        internal static /* Tuple */ IntPtr dotnet_to_rust(Tuple<float, float> tuple)
        {
            var t_1 = tuple.Item1;
            var t_1_rust = t_1;
            var t_2 = tuple.Item2;
            var t_2_rust = t_2;
            // We don't call delete in `Input` scenario. Rust-side conversion code will take care of it.
            return RustTuple2Tfloatfloat_new(t_1_rust, t_2_rust);            
        }
    }
    

        public class Option<T> {
        
            [System.Serializable]
            public class OptionNoneException : System.Exception
            {
                public OptionNoneException() :
                    base("Trying to get the value of an `Option` that is `None`") 
                {
                }
            }
        
            private T value;
            private bool isSome;
        
            public bool IsSome
            {
                get
                {
                    return isSome;
                }
            }
        
            public T Value
            {
                get {
                    if (!isSome) {
                        throw new OptionNoneException();
                    }
                    return value;
                }
            }
        
            public Option()
            {
                value = default(T);
                isSome = false;
            }
        
            public Option(T value)
            {
                if (value == null) 
                {
                    this.value = value;
                    this.isSome = false;
                }
                else
                {
                    this.value = value;
                    this.isSome = true;
                }
            }
        }        
        
    public static class RustVecstring {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustVecstring_new();
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecstring_push(IntPtr vecPtr, /* RustString */ IntPtr element);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option<i_type> */ IntPtr RustVecstring_iter_next(IntPtr iterPtr);
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecstring_iter_delete(IntPtr iterPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* RustString */ IntPtr RustVecstring_option_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustVecstring_option_is_some(IntPtr optPtr);


        internal static System.Collections.Generic.List<string> rust_to_dotnet(IntPtr iterPtr) {
            var list = new System.Collections.Generic.List<string>();
            while (true)
            {
                var next_rust_opt = RustVecstring.RustVecstring_iter_next(iterPtr);
                if (RustVecstring_option_is_some(next_rust_opt) == 0)
                {
                    break;
                }
                var value_rust = RustVecstring_option_take(next_rust_opt);
                var value = RustString.rust_to_dotnet(value_rust);
                list.Add(value);
            }
            RustVecstring_iter_delete(iterPtr);
            return list;
        }

        internal static IntPtr dotnet_to_rust(System.Collections.Generic.List<string> list) {
            var vec = RustVecstring_new();
            foreach (var element in list)
            {
                var i_element = RustString.dotnet_to_rust(element);
                RustVecstring.RustVecstring_push(vec, i_element);
            }
            return vec;
        }
    }
        
    public static class RustVecJumpData {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustVecJumpData_new();
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecJumpData_push(IntPtr vecPtr, /* JumpData */ IntPtr element);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option<i_type> */ IntPtr RustVecJumpData_iter_next(IntPtr iterPtr);
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecJumpData_iter_delete(IntPtr iterPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* JumpData */ IntPtr RustVecJumpData_option_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustVecJumpData_option_is_some(IntPtr optPtr);


        internal static System.Collections.Generic.List<JumpData> rust_to_dotnet(IntPtr iterPtr) {
            var list = new System.Collections.Generic.List<JumpData>();
            while (true)
            {
                var next_rust_opt = RustVecJumpData.RustVecJumpData_iter_next(iterPtr);
                if (RustVecJumpData_option_is_some(next_rust_opt) == 0)
                {
                    break;
                }
                var value_rust = RustVecJumpData_option_take(next_rust_opt);
                var value = new JumpData(value_rust);
                list.Add(value);
            }
            RustVecJumpData_iter_delete(iterPtr);
            return list;
        }

        internal static IntPtr dotnet_to_rust(System.Collections.Generic.List<JumpData> list) {
            var vec = RustVecJumpData_new();
            foreach (var element in list)
            {
                var i_element = element.nativePtr;
                RustVecJumpData.RustVecJumpData_push(vec, i_element);
            }
            return vec;
        }
    }
        
    internal static class RustOptionObjData {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustOptionObjData_new_none();

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustOptionObjData_new_some(/* ObjData */ IntPtr value);
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* ObjData */ IntPtr RustOptionObjData_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustOptionObjData_is_some(IntPtr optPtr);

        internal static Option<ObjData> rust_to_dotnet(IntPtr optPtr)
        {
            if (RustOptionObjData_is_some(optPtr) != 0)
            {
                var value_0 = RustOptionObjData_take(optPtr);
                var value_1 = new ObjData(value_0);
                return new Option<ObjData>(value_1);
            }
            else
            {
                return new Option<ObjData>();
            }
        }

        internal static IntPtr dotnet_to_rust(Option<ObjData> opt)
        {
            if (opt.IsSome)
            {
                var value_0 = opt.Value.nativePtr;
                return RustOptionObjData_new_some(value_0);
            }
            else
            {
                return RustOptionObjData_new_none();
            }
        }
    }
    
    public static class RustVecEventData {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustVecEventData_new();
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecEventData_push(IntPtr vecPtr, /* EventData */ IntPtr element);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option<i_type> */ IntPtr RustVecEventData_iter_next(IntPtr iterPtr);
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecEventData_iter_delete(IntPtr iterPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* EventData */ IntPtr RustVecEventData_option_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustVecEventData_option_is_some(IntPtr optPtr);


        internal static System.Collections.Generic.List<EventData> rust_to_dotnet(IntPtr iterPtr) {
            var list = new System.Collections.Generic.List<EventData>();
            while (true)
            {
                var next_rust_opt = RustVecEventData.RustVecEventData_iter_next(iterPtr);
                if (RustVecEventData_option_is_some(next_rust_opt) == 0)
                {
                    break;
                }
                var value_rust = RustVecEventData_option_take(next_rust_opt);
                var value = new EventData(value_rust);
                list.Add(value);
            }
            RustVecEventData_iter_delete(iterPtr);
            return list;
        }

        internal static IntPtr dotnet_to_rust(System.Collections.Generic.List<EventData> list) {
            var vec = RustVecEventData_new();
            foreach (var element in list)
            {
                var i_element = element.nativePtr;
                RustVecEventData.RustVecEventData_push(vec, i_element);
            }
            return vec;
        }
    }
        
    public static class RustVecSectorData {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustVecSectorData_new();
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecSectorData_push(IntPtr vecPtr, /* SectorData */ IntPtr element);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option<i_type> */ IntPtr RustVecSectorData_iter_next(IntPtr iterPtr);
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecSectorData_iter_delete(IntPtr iterPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* SectorData */ IntPtr RustVecSectorData_option_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustVecSectorData_option_is_some(IntPtr optPtr);


        internal static System.Collections.Generic.List<SectorData> rust_to_dotnet(IntPtr iterPtr) {
            var list = new System.Collections.Generic.List<SectorData>();
            while (true)
            {
                var next_rust_opt = RustVecSectorData.RustVecSectorData_iter_next(iterPtr);
                if (RustVecSectorData_option_is_some(next_rust_opt) == 0)
                {
                    break;
                }
                var value_rust = RustVecSectorData_option_take(next_rust_opt);
                var value = new SectorData(value_rust);
                list.Add(value);
            }
            RustVecSectorData_iter_delete(iterPtr);
            return list;
        }

        internal static IntPtr dotnet_to_rust(System.Collections.Generic.List<SectorData> list) {
            var vec = RustVecSectorData_new();
            foreach (var element in list)
            {
                var i_element = element.nativePtr;
                RustVecSectorData.RustVecSectorData_push(vec, i_element);
            }
            return vec;
        }
    }
        
    public static class RustVecObjData {
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr RustVecObjData_new();
        
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecObjData_push(IntPtr vecPtr, /* ObjData */ IntPtr element);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* Option<i_type> */ IntPtr RustVecObjData_iter_next(IntPtr iterPtr);
        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void RustVecObjData_iter_delete(IntPtr iterPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern /* ObjData */ IntPtr RustVecObjData_option_take(IntPtr optPtr);

        [DllImport("ffi_domain_2_native", CallingConvention = CallingConvention.Cdecl)]
        internal static extern byte RustVecObjData_option_is_some(IntPtr optPtr);


        internal static System.Collections.Generic.List<ObjData> rust_to_dotnet(IntPtr iterPtr) {
            var list = new System.Collections.Generic.List<ObjData>();
            while (true)
            {
                var next_rust_opt = RustVecObjData.RustVecObjData_iter_next(iterPtr);
                if (RustVecObjData_option_is_some(next_rust_opt) == 0)
                {
                    break;
                }
                var value_rust = RustVecObjData_option_take(next_rust_opt);
                var value = new ObjData(value_rust);
                list.Add(value);
            }
            RustVecObjData_iter_delete(iterPtr);
            return list;
        }

        internal static IntPtr dotnet_to_rust(System.Collections.Generic.List<ObjData> list) {
            var vec = RustVecObjData_new();
            foreach (var element in list)
            {
                var i_element = element.nativePtr;
                RustVecObjData.RustVecObjData_push(vec, i_element);
            }
            return vec;
        }
    }
        } // namespace

using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using utils;

namespace game.domain
{

    public class GenericObject : MonoBehaviour
    {
        public Id id;
        public ObjKind kind;

        public void UpdateName()
        {
            var kindName = Enum.GetName(typeof(ObjKind), this.kind);
            gameObject.name = kindName + " (" + id.value + ")";
        }

        public void Hide()
        {
            gameObject.GetComponent<DrawGizmos>().draw = false;
        }

        public void Show()
        {
            gameObject.GetComponent<DrawGizmos>().draw = true;
        }
    }
}